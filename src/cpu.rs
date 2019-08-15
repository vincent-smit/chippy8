const WIDTH: usize = 64;
const HEIGHT: usize = 32;

use std::convert::TryInto;
use rand;


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
    pub draw_flag: bool,
}


impl CPU {

    pub fn new(binary: Vec<u8>, fsize: u64) -> CPU {

        let mut memory = [0;4096];
        if (4096 - 512) > fsize
        {
            for i in 0..fsize
            {
                memory[(i + 512) as usize] = binary[i as usize];
            }
        }
        else
        {
            panic!("ROM too big for memory");
        }

        let mut register = Vec::with_capacity(16);
        for _nr in 0..16 {
            register.push(0);
        };
        let mut stack = Vec::with_capacity(16);

        for _nr in 0..16 {
            stack.push(0);
        }

        let fontset: [u8;80] = 
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
            pixels.push((0,0,0));
        };

        CPU {
                memory : memory.to_vec(),
                register : register,
                sound_register : 0,
                delay_register : 0,
                pc : 0x200,
                sp : 0,
                stack : stack,
                i : 0,
                keyboard: keyboard,
                pixels: pixels,
                draw_flag: true
            }
    }
    
    pub fn fetch_opcode(&mut self) -> Result<u16, ()> {

        let opcode: u16 = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;


        println!("Parsing.. {:x} (in hex) / {:b} (in binary)",opcode, opcode);
        //println!("First nibble '{:b}', second nibble '{:b}', third nibble '{:b}', fourth nibble '{:b}'", opcode & 0xF000,opcode & 0x0F00, opcode & 0x00F0, opcode & 0x000F);
        //println!("First nibble '{:x}', second nibble '{:x}', third nibble '{:x}', fourth nibble '{:x}'", opcode & 0xF000,opcode & 0x0F00, opcode & 0x00F0, opcode & 0x000F );

        Ok(opcode)
    }

    pub fn parse_instruction(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        println!("Clear screen");
                        for xy in 0..WIDTH*HEIGHT {
                            self.pixels[xy as usize] = (255,255,255);
                        }   
                        self.draw_flag = true;
                        self.pc += 2;
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
                self.pc = (opcode & 0x0FFF) as usize;
            }

            0x2000 => {
                println!("Calls subroutine at NNN");
                self.stack[self.sp] = self.pc +2;
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
                let pos: usize = ((opcode & 0x0F00) >> 8) as usize;
                self.register[pos] = self.register[pos].wrapping_add((opcode & 0x00FF) as u8);
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
                        self.register[((opcode & 0x0F00) >> 8) as usize] |= self.register[((opcode & 0x00F0) >> 4) as usize];
                    }

                    0x0002 => {
                        println!("Vx=Vx&Vy");
                        self.register[((opcode & 0x0F00) >> 8) as usize] &= self.register[((opcode & 0x00F0) >> 4) as usize];
                    }
                    0x0003 => {
                        println!("Vx=Vx^Vy");
                        self.register[((opcode & 0x0F00) >> 8) as usize] ^= self.register[((opcode & 0x00F0) >> 4) as usize];
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
                        println!("Vx>>=1");
                        self.register[0xF] = self.register[((opcode & 0x0F00) >>8) as usize] & 0x1;
                        self.register[((opcode & 0x0F00) >>8) as usize] >>=1;
                    }
                    0x007 => {
                        println!("Vy -= Vx");
                        let (val, ovf) = self.register[((opcode & 0x00F0)  >> 4) as usize].overflowing_sub(self.register[((opcode & 0x0F00) >> 8) as usize]);
                        self.register[((opcode & 0x00F0)  >> 4) as usize] = val;
                        self.register[0xF] = ovf as u8;
                    }
                    // CHECK
                    0x000E => {
                        println!("Vx<<=1");        
                        self.register[0xF] = self.register[((opcode & 0x0F00) >>8) as usize] >>7;
                        self.register[((opcode & 0x0F00) >>8) as usize] <<=1;
                    } 
                    _ => {
                        panic!("Unknown opcode [0x8000] {:x}", opcode);
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
                self.i = (opcode & 0x0FFF) as u8;
                self.pc += 2;
            }

            0xB000 => {
                println!("PC=V0+NNN");
                let target = ((opcode & 0x0FFF) as u8).wrapping_add(self.register[0]);
                self.pc = target as usize;
            }
            
            0xC000 => {
                self.register[((opcode & 0x0F00) >> 8) as usize] = rand::random::<u8>() & (opcode as u8);
                println!("Vx=rand()&NN");
                self.pc += 2;
            }
            
            0xD000 => {
                println!("draw(Vx,Vy,HEIGHT)");
                
                let x = self.register[((opcode & 0x0F00) >> 8) as usize] as u16;
                let y = self.register[((opcode & 0x00F0) >> 4) as usize] as u16;
                let height: u16 = ((opcode & 0x000F)).try_into().unwrap();

                println!("{:x}",x);
                println!("{:x}",y);
                println!("{}",height);

                self.register[0xF] = 0;
                for yline in 0..height {
                    let pixel = self.memory[(self.i as u16+yline) as usize] as u16;
                    for xline in 0..8 {
                        if (pixel & 0x80 >> xline) != 0 {
                            //let location: u16 = ((xline + x)*(yline + y)).into();
                            let pos = ((x + xline) + ((y + yline) * 64)) as usize;
                            if pos < 2048 {
                                if self.pixels[pos as usize] == (255,255,255) {
                                    self.pixels[pos as usize] = (0,0,0);
                                    self.register[0xF] = 1;
                                }
                                else {
                                    self.pixels[pos as usize] = (255,255,255);
                                }
                            }
                        }
                    }
                }
                
                self.pc += 2;
                self.draw_flag = true;
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
                        if self.keyboard.key[((opcode & 0x0F00) >> 8) as usize] == 0 {
                            self.pc += 4;
                        }
                        else {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        panic!("E not implemented!")
                    }
                }
            }
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        println!("Vx = get_delay()");
                        self.register[(opcode & 0x0F00 >> 8) as usize] = self.delay_register;
                        self.pc += 2;

                    },
                    0x000A => {
                        let mut key_press = false;
                        println!("Vx = get_key()");
                        for i in 0..16 {
                            if self.keyboard.key[i as usize] != 0 {
                                self.register[((opcode & 0x0F00) >>8) as usize] = i;
                                key_press = true;
                            }
                            if key_press {
                                self.pc +=2;
                            }
                        }
                        self.pc +=2

                    },
                    0x0015 => {
                        println!("delay_timer(Vx)");
                        self.delay_register = self.register[(opcode & 0x0F00 >> 8) as usize];
                        self.pc +=2
                    },
                    0x0018 => {
                        println!("sound_timer(Vx)");
                        self.sound_register = self.register[(opcode & 0x0F00 >> 8) as usize];
                        self.pc +=2;
                    },
                    0x001E => {
                        println!("I +=Vx");
                        let (val, ovf) = self.i.overflowing_add(self.register[((opcode & 0x0F00) >> 8) as usize]);
                        self.register[((opcode & 0x0F00)  >> 8) as usize] = val;
                        self.register[0xF] = ovf as u8;
                        self.pc +=2
                    },
                    0x0029 => {
                        println!("I=sprite_addr[Vx]");
                        self.i = self.register[((opcode & 0x0F00) >>8) as usize] *0x5;
                        self.pc +=2;
                    },
                    0x0033 => {
                        println!("set_BCD(Vx)");
                        self.memory[(self.i) as usize]     = self.register[((opcode & 0x0F00) >> 8) as usize] / 100;
					    self.memory[(self.i + 1) as usize] = (self.register[((opcode & 0x0F00) >> 8) as usize] / 10) % 10;
					    self.memory[(self.i + 2) as usize] = (self.register[((opcode & 0x0F00) >> 8) as usize] % 100) % 10;	
                        self.pc +=2;
                    },
                    0x0055 => {
                        println!("reg_dump(Vx,&I)");
                        let j = ((opcode & 0x0F00) >> 8) as u8; 

                        for i in 0..j+1 {
                            self.memory[(self.i+i) as usize] = self.register[i as usize];
                        }
                        self.i = self.i.wrapping_add(j+1).try_into().unwrap();
                        self.pc +=2;
                    },
                    0x0065 => {
                        println!("reg_load(Vx,&I)");
                        let j = ((opcode & 0x0F00) >> 8) as u8;

                        for i in 0..j+1{
                            self.register[i as usize] = self.memory[(self.i + i as u8) as usize];
                        }
                        self.i = self.i.wrapping_add(j+1).try_into().unwrap();
                        self.pc +=2;
                    }
                    _ => {
                        panic!("F not implemented!");
                    }
                }
            }
            _ => {
                panic!("Opcode not implemented!")
            }
        }
    }
}