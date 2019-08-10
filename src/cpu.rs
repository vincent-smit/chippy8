const WIDTH: usize = 64;
const HEIGHT: usize = 32;

use rand::Rng;
use std::convert::TryInto;

pub struct Keyboard {
    key: Vec<u8>
}

pub struct CPU {
    pub memory: Vec<u8>,
    pub register: Vec<u8>,
    pub sound_register: u8,
    pub delay_register: u8,
    pub pc: usize,
    pub sp: usize,
    pub stack: Vec<usize>,
    pub i: u8,
    pub keyboard: Keyboard,
    pub pixels: Vec<(u8,u8,u8)>,
}


impl CPU {

    pub fn new(binary: Vec<u8>) -> CPU {

        let mut memory: Vec<u8> = Vec::with_capacity(4096);

        let mem_start: u16 = 512;
        let value = 0;

        for _i in 0..mem_start {
            memory.push(value.clone());
        }

        for (_i, hex) in binary.iter().enumerate() {
            memory.push(hex.clone())
        }
        println!("{:?}", memory);

        let mut register = Vec::with_capacity(16);
        for _nr in 0..16 {
            register.push(0);
        };
        let mut stack = Vec::with_capacity(16);

        for _nr in 0..16 {
            stack.push(0);
        }

        let fontset = 
        [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        // Load fontset
        for i in 0..80 {
            memory[i] = fontset[i];
        };

        let mut keyboard = Keyboard {key: Vec::new()};
        for _i in 0..16 {
            keyboard.key.push(0)
        }

        let mut pixels = vec![];
        for _i in 0..(WIDTH*HEIGHT) {
            pixels.push((255,255,255));
        };

        CPU {
                memory : memory,
                register : register,
                sound_register : 0,
                delay_register : 0,
                pc : 0x200,
                sp : 0,
                stack : stack,
                i : 0,
                keyboard: keyboard,
                pixels: pixels
            }
    }
    
    pub fn fetch_opcode(&mut self) -> Result<u16, ()> {
        let left: u16 = self.memory[self.pc].clone().into();
        let right: u16 = self.memory[self.pc + 1].clone().into();

        let opcode: u16 = (left << 8| right) as u16;

        println!("Parsing.. {:x} (in hex) / {:b} (in binary)",opcode, opcode);
        println!("First nibble '{:b}', second nibble '{:b}', third nibble '{:b}', fourth nibble '{:b}'", opcode & 0xF000,opcode & 0x0F00, opcode & 0x00F0, opcode & 0x000F);
        println!("First nibble '{:x}', second nibble '{:x}', third nibble '{:x}', fourth nibble '{:x}'", opcode & 0xF000,opcode & 0x0F00, opcode & 0x00F0, opcode & 0x000F );

        Ok(opcode)
    }

    pub fn parse_instruction(&mut self, opcode: u16) -> Option<Vec<(u8,u8,u8)>> {

        println!("{}", opcode & 0xF000);
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        println!("Clear screen");
                        for xy in 0..WIDTH*HEIGHT {
                            self.pixels[xy as usize] = (0,0,0);
                        }   
                
                        let grid = self.pixels.clone();
                        self.pc += 2;
                        return Some(grid);
                    }

                    0x000E => {
                        println!("Returns from a subroutine");
                        self.sp -= 1;
                        self.pc =  self.stack[self.sp];
                        self.pc += 2;
                    }
                    _ => {
                        println!("Opcode not recorgnized. {}", opcode);
                    }
                }
            }

            0x1000 => {
                println!("JUMP NNN, e.g. {:x}", (opcode & 0xFFF));
                self.pc = (opcode & 0xFFF) as usize;
            }

            0x2000 => {
                println!("Calls subroutine at NNN");
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = (opcode & 0x0FFF) as usize;
            }

            0x3000 => {
                if self.register[((opcode & 0x0F00) >> 8) as usize] == (opcode & 0x00FF) as u8 {
                    println!("SKIP.EQ");
                    self.pc += 4;
                }
                else {
                    println!("NOT SKIP.EQ");
                    self.pc += 2;
                }               
            }
            
            0x4000 => {
                if self.register[((opcode & 0x0F00) >> 8) as usize] != (opcode & 0x00FF) as u8 {
                    println!("SKIP.NE");
                    self.pc += 4;
                }
                else {
                    println!("NOT SKIP.NE");
                    self.pc += 2;
                }
            }

            0x5000 => {
                if self.register[((opcode & 0x0F00) >> 8) as usize] == self.register[((opcode & 0x00F0) >> 4) as usize] {
                    println!("SKIP.EQ");
                    self.pc += 4;
                }
                else {
                    println!("NOT SKIP.EQ");
                    self.pc += 2;
                } 
            }
            
            0x6000 => {
               println!("Vx = NN");
                self.register[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
                self.pc += 2;
            }

            0x7000 => {
                println!("Adds NN to VX");
                println!("{}", self.register[((opcode & 0x0F00) >> 8) as usize] );
                println!("{}", (opcode & 0x00FF) as u8);
                self.register[((opcode & 0x0F00) >> 8) as usize] = self.register[((opcode & 0x0F00) >> 8) as usize].wrapping_add((opcode & 0x00FF) as u8);
                self.pc += 2;
            }

            0x8000 => {
                self.pc += 2;
                match opcode & 0x000F {
                    0x0000 => {
                        println!("Vx=Vy");
                        self.register[((opcode & 0x0F00) >> 8) as usize] = self.register[((opcode & 0x00F0) >> 4) as usize];
                    }

                    0x0001 => {
                        println!("Vx=Vx|Vy");
                        self.register[((opcode & 0x0F00) >> 8) as usize] = self.register[((opcode & 0x0F00) >> 8) as usize] | self.register[((opcode & 0x00F0) >> 4) as usize];
                    }

                    0x0002 => {
                        println!("Vx=Vx&Vy");
                        self.register[((opcode & 0x0F00) >> 8) as usize] = self.register[((opcode & 0x0F00) >> 8) as usize] & self.register[((opcode & 0x00F0) >> 4) as usize];
                    }
                    0x0003 => {
                        println!("Vx=Vx^Vy");
                        self.register[((opcode & 0x0F00) >> 8) as usize] = self.register[((opcode & 0x0F00) >> 8) as usize] ^ self.register[((opcode & 0x00F0) >> 4) as usize];
                    }
                    // TODO: Check 
                    0x0004 => {
                        println!("Vx += Vy");
                        let (val, ovf) = self.register[((opcode & 0x0F00)  >> 8) as usize].overflowing_add(self.register[((opcode & 0x00F0) >> 4) as usize]);
                        self.register[((opcode & 0x0F00)  >> 8) as usize] = val;
                        self.register[0xF] = ovf as u8;
                    }
                    0x0005 => {
                        println!("Vx -= Vy");
                        let (val, ovf) = self.register[((opcode & 0x0F00)  >> 8) as usize].overflowing_sub(self.register[((opcode & 0x00F0) >> 4) as usize]);
                        self.register[((opcode & 0x0F00)  >> 8) as usize] = val;
                        self.register[0xF] = ovf as u8;
                    }
                    // TODO: Check
                    0x0006 => {
                        println!("{:b}",opcode & 0x0F00 >> 1);
                        println!("Vx>>=1");
                        self.register[0xF] = self.register[((opcode & 0x0F00) >>8) as usize] >>1;
                        self.register[((opcode & 0x0F00) >>8) as usize] >>=1;
                    }
                    0x007 => {
                        println!("Vy -= Vx");
                        let (val, ovf) = self.register[((opcode & 0x00F0)  >> 4) as usize].overflowing_sub(self.register[((opcode & 0x0F00) >> 8) as usize]);
                        self.register[((opcode & 0x0F0)  >> 4) as usize] = val;
                        self.register[0xF] = ovf as u8;
                    }
                    // CHECK
                    0x000E => {
                        println!("{:b}",opcode & 0x0F00 << 1);
                        println!("Vx<<=1");        
                        self.register[0xF] = self.register[((opcode & 0x0F00) >>8) as usize] >>7;
                        self.register[((opcode & 0x0F00) >>8) as usize] <<=1;
                    } 
                    _ => {
                        println!("Unknown opcode [0x8000] {:x}", opcode);
                    }
                }
            }
            
            0x9000 => {
                println!("if(Vx!=Vy)");
                if self.register[((opcode & 0x0F00) >> 8) as usize] != self.register[((opcode & 0x00F0) >> 4) as usize] {
                    self.pc += 4;
                }
                else {
                    self.pc += 2;
                }
            }
            
            0xA000 => {
                println!("I = NNN");
                println!("{:x}",opcode & 0x0FFF);
                self.i = (opcode & 0x0FFF) as u8;
                self.pc += 2;
            }

            0xB000 => {
                println!("PC=V0+NNN");
                let target = (opcode & 0x0FFF) as u8 + self.register[0x0];
                self.pc = target as usize;
            }
            
            0xC000 => {
                self.register[((opcode & 0x0F00) >> 8) as usize] = (rand::thread_rng().gen_range(0, 255) as u8) & (opcode & 0xFF) as u8;
                println!("Vx=rand()&NN");
                self.pc += 2;
            }
            
            0xD000 => {
                println!("draw(Vx,Vy,HEIGHT)");
                
                let x = self.register[((opcode & 0x0F00) >> 8) as usize];
                let y = self.register[((opcode & 0x00F0) >> 4) as usize];
                let height: u8 = ((opcode & 0x000F)).try_into().unwrap();

                println!("{}",x);
                println!("{}",y);
                println!("{}",height);

                for yline in 0..height -1{
                    let pixel = self.memory[(self.i+yline) as usize];
                    for xline in 0..7 {
                        if (pixel & 0x080 >> xline) != 0 {
                            let location: u16 = ((xline as u16 + x as u16)*(yline as u16 + y as u16)).into();
                            if self.pixels[location as usize] == (255,255,255) {
                                self.pixels[location as usize] = (0,0,0);
                                self.register[0xF] = 1;
                            }
                            else {
                                self.pixels[location as usize] = (255,255,255);
                            }
                        }
                    }
                }
                
                let grid = self.pixels.clone();
                self.pc += 2;
                return Some(grid);
            }

            // TODO: Keys
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        println!("if(key()==Vx)");
                        if self.keyboard.key[((opcode & 0x0F00) >> 8) as usize] != 0 {
                            self.pc += 4;
                        }
                        else {
                            self.pc += 2;
                        }
                    }

                    0x00A1 => {
                        println!("if(key()!=Vx)");
                        if self.keyboard.key[((opcode & 0x0F00) >> 8) as usize] != 1 {
                            self.pc += 4;
                        }
                        else {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        println!("E not implemented!")
                    }
                }
            }

            // TODO Check this

            0xF000 => {
                self.pc += 2;
                println!("{:x}", opcode & 0x00F0);
                match (opcode & 0x00F0, opcode & 0x000F) {
                    (0x0000,0x0007) => {
                        println!("Vx = get_delay()");
                        self.register[(opcode & 0x0F00 >> 8) as usize] = self.delay_register;
                    },
                    (0x000,0x000A) => {
                        println!("Vx = get_key()");
                        for i in 0..15 {
                            if self.keyboard.key[i as usize] != 0 {
                                self.register[((opcode & 0x0F00) >>8) as usize];
                            }
                        }
                    },
                    (0x0010,0x0005) => {
                        println!("delay_timer(Vx)");
                        self.delay_register = self.register[(opcode & 0x0F00 >> 8) as usize];
                    },
                    (0x0010,0x0008) => {
                        println!("sound_timer(Vx)");
                        self.sound_register = self.register[(opcode & 0x0F00 >> 8) as usize];
                    },
                    (0x0010,0x000E) => {
                        println!("I +=Vx, check");
                        println!("{} + {}", self.i, self.register[(opcode & 0x0F00 >> 8) as usize]);
                    },
                    (0x0020,0x009) => {
                        println!("I=sprite_addr[Vx]");
                        self.i = self.register[((opcode & 0x0F00) >>8) as usize] * 0x5;
                    },
                    (0x0030,0x0003) => {
                        println!("set_BCD(Vx)");
                        self.memory[(self.i) as usize]     = self.register[((opcode & 0x0F00) >> 8) as usize] / 100;
					    self.memory[(self.i + 1) as usize] = (self.register[((opcode & 0x0F00) >> 8) as usize] / 10) % 10;
					    self.memory[(self.i + 2) as usize] = (self.register[((opcode & 0x0F00) >> 8) as usize] % 100) % 10;	
                    },
                    (0x0050,0x0005) => {
                        println!("reg_dump(Vx,&I)");
                        for i in 0..(self.register[(opcode & 0x0F00 >> 8) as usize]) {
                            self.memory[(self.i+i) as usize] = self.register[i as usize];
                        }
                    },
                    (0x0060,0x0005) => {
                        println!("reg_load(Vx,&I)");
                        for i in 0..(opcode & 0x0F00 >>8) {
                            self.register[i as usize] = self.memory[(self.i + i as u8) as usize];
                        }
                    }
                    (_,_) => {
                        println!("F not implemented!")
                    }
                }
            }
            _ => {
                println!("Everything else")
            }
        }

        None
    }
}