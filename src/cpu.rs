const M: usize = 256;
const N: usize = 256;

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
    pub pixels: Vec<u8>
}


impl CPU {

    pub fn new(binary: Vec<u8>) -> CPU {

        let mut memory: Vec<u8> = Vec::with_capacity(4096);

        let mem_start: u16 = 512;
        let value = 0;

        for i in 0..mem_start {
            memory.push(value.clone());
        }

        for (i, hex) in binary.iter().enumerate() {
            memory.push(hex.clone())
        }
        println!("{:?}", memory);

        let mut register = Vec::with_capacity(16);
        for nr in 0..16 {
            register.push(nr);
        };
        let mut stack = Vec::with_capacity(16);

        for nr in 0..16 {
            stack.push(nr);
        }

        let mut fontset: Vec<u8> = Vec::new();

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

        for i in 0..16 {
            keyboard.key.push(0)
        }

        let mut pixels = vec![0; M * N];

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
        println!("First nibble '{:b}', second nibble '{:b}', third nibble '{:b}', fourth nibble '{:b}'", opcode & 0xFFFF,opcode & 0x0FFF, opcode & 0x00FF, opcode & 0x0F );
        println!("First nibble '{:x}', second nibble '{:x}', third nibble '{:x}', fourth nibble '{:x}'", opcode & 0xFFFF,opcode & 0x0FFF, opcode & 0x00FF, opcode & 0x0F );

        Ok(opcode)
    }

    pub fn parse_instruction(&mut self, opcode: u16) -> Option<Vec<u8>> {

        println!("{}", opcode & 0xF000);
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0xF {
                    0 => {
                        println!("Clear screen");
                        self.pc += 2;
                        Some("clear".to_owned());
                    }
                    _ => {
                        println!("Returns from a subroutine");
                        self.pc =  self.stack[self.sp]
                    }
                }
            }

            0x1000 => {
                println!("JUMP NNN, e.g. {:x}", (opcode & 0xFFF));
                //self.stack[self.sp] = self.pc;
                //self.sp += 1;
                self.pc = (opcode & 0xFFF) as usize;
            }

            0x2000 => {
                println!("Calls subroutine at NNN");
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = (opcode & 0x0FFF) as usize;
            }

            0x3000 => {
                if self.register[((opcode & 0xF00) >> 8) as usize] == (opcode & 0xFF) as u8 {
                    println!("SKIP.EQ");
                    self.pc += 4;
                }
                else {
                    println!("NOT SKIP.EQ");
                    self.pc += 2;
                }               
            }
            
            0x4000 => {
                if self.register[((opcode & 0xF00) >> 8) as usize] != (opcode & 0xFF) as u8 {
                    println!("SKIP.NE");
                    self.pc += 4;
                }
                else {
                    println!("NOT SKIP.NE");
                    self.pc += 2;

                }
            }

            0x5000 => {
                if self.register[((opcode & 0xF00) >> 8) as usize] == self.register[((opcode & 0xF0) >> 4) as usize] {
                    println!("SKIP.EQ");
                    self.pc += 4;
                }
                else {
                    println!("NOT SKIP.EQ");
                    self.pc += 2;
                } 
            }
            
            0x6000 => {
                println!("{}", self.register[((opcode & 0x0F00) >>8) as usize]);
                self.register[((opcode & 0xF00) >> 8) as usize] = (opcode & 0xFF) as u8;
                println!("Vx = NN");
                self.pc += 2;
            }

            0x7000 => {
                self.register[((opcode & 0xF00) >> 8) as usize] = (opcode & 0xFF) as u8;
                println!("Adds NN to VX");
                self.pc += 2;

            }

            0x8000 => {
                self.pc += 2;
                match opcode & 0xF {
                    0 => {
                        println!("Vx=Vy");
                        self.register[((opcode & 0xF00) >> 8) as usize] = self.register[((opcode & 0xF0) >> 4) as usize];
                    }

                    1 => {
                        println!("Vx=Vx|Vy");
                        self.register[((opcode & 0xF00) >> 8) as usize] = self.register[((opcode & 0xF00) >> 8) as usize] | self.register[((opcode & 0xF0) >> 4) as usize];
                    }

                    2 => {
                        println!("Vx=Vx&Vy");
                        self.register[((opcode & 0xF00) >> 8) as usize] = self.register[((opcode & 0xF00) >> 8) as usize] & self.register[((opcode & 0xF0) >> 4) as usize];
                    }
                    3 => {
                        println!("Vx=Vx^Vy");
                        self.register[((opcode & 0xF00) >> 8) as usize] = self.register[((opcode & 0xF00) >> 8) as usize] ^ self.register[((opcode & 0xF0) >> 4) as usize];
                    }
                    // TODO: Check 
                    4 => {
                        println!("Vx += Vy");
                        // 13 > 255-100= 155
                        if self.register[(opcode & 0x00F0) as usize  >> 4] > (0xFF - self.register[(opcode & 0x0F00) as usize >> 8]) {
                            self.register[0xF] = 1; //carry
                        }
                        else {
                            self.register[0xF] = 0;
                            self.register[(opcode & 0x0F00) as usize >> 8] += self.register[(opcode & 0x00F0) as usize >> 4];
                        }
                    }
                    5 => {
                        println!("Vx -= Vy");
                        if self.register[(opcode & 0x00F0) as usize] >= self.register[(opcode & 0x0F00) as usize] {
                            self.register[0xF] = 1;
                        }
                        else {
                            self.register[0xF] = 0; //carry
                        }
                        let mut result = self.register[(opcode & 0x0F00) as usize] - self.register[(opcode & 0x00F0) as usize];
                        if result < 0 {
                            result +=255
                        };
                        self.register[(opcode & 0x00F0) as usize] = result;

                    }
                    // TODO: Check
                    6 => {
                        println!("{:b}",opcode & 0x0F00 >> 1);
                        println!("Vx>>=1");
                        self.register[0xF] = self.register[(opcode & 0x0F00) as usize]  >>1
                    }
                    7 => {
                        println!("Vx=Vy-Vx");
                        println!("Vx -= Vy");
                        if self.register[(opcode & 0x0F00) as usize] >= self.register[(opcode & 0x00F0) as usize] {
                            self.register[0xF] = 1;
                        }
                        else {
                            self.register[0xF] = 0; //carry
                        }
                        let mut result = self.register[(opcode & 0x00F0) as usize] - self.register[(opcode & 0x0F00) as usize];
                        if result < 0 {
                            result +=255
                        };
                        self.register[(opcode & 0x00F0) as usize] = result;
                    }
                    // CHECK
                    14 => {
                        println!("{:b}",opcode & 0x0F00 << 1);
                        println!("Vx<<=1");
                        self.register[0xF] = self.register[(opcode & 0x0F00) as usize]  <<1
                    } 
                    _ => {
                        println!("Else")
                    }
                }
            }
            
            0x9000 => {
                println!("if(Vx!=Vy)");
                if self.register[((opcode & 0xF00) >> 8) as usize] != self.register[((opcode & 0xF0) >> 4) as usize] {
                    self.pc += 4;
                }
                else {
                    self.pc += 2;
                }
            }
            
            0xA000 => {
                println!("I = NNN");
                println!("{:x}",opcode & 0xFFF);
                self.i = (opcode & 0xFFF) as u8;
                self.pc += 2;
            }

            0xB000 => {
                println!("PC=V0+NNN");
                let target = (opcode & 0xFFF) as u8 + self.register[0x0];
                self.pc = target as usize;
                self.pc += 2;
            }
            
            0xC000 => {
                self.register[((opcode & 0xF00) >> 8) as usize] = (rand::thread_rng().gen_range(0, 255) as u8) & (opcode & 0xFF) as u8;
                println!("Vx=rand()&NN");
                self.pc += 2;
            }
            
            // TODO: Draw sprite
            0xD000 => {
                println!("draw(Vx,Vy,N)");
                //self.register[0xF] = 0;
                let x = self.register[((opcode & 0x0F00) >> 8) as usize];
                let y = self.register[((opcode & 0x00F0) >> 4) as usize];
                let height: u8 = ((opcode & 0x000F)).try_into().unwrap();


                for line in 0..height {
                    let draw = self.memory[(self.i + line) as usize];
                    for i in 0..200 {
                        //if draw & (0x80 >> i) != 0 {
                            let mut nr = ((line+x as u8) * (M as u8) + (N as u8 + i));
                            println!("{}", nr);
                            self.pixels[((line+x) * (M as u8) + ((N as u8)+ i)) as usize] = 255
                        //}
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
                        //self.i += self.register[(opcode & 0x0F00 >> 8) as usize];
                    },
                    (0x0020,0x009) => {
                        println!("I=sprite_addr[Vx]")
                    },
                    (0x0030,0x0003) => {
                        println!("set_BCD(Vx)")
                    },
                    (0x0050,0x0005) => {
                        println!("reg_dump(Vx,&I)");
                        for i in 0..(self.register[(opcode & 0x0F00 >> 8) as usize]) {
                            self.memory[(self.i+i) as usize] = self.register[i as usize];
                        }
                    },
                    (0x0060,0x0005) => {
                        println!("reg_load(Vx,&I)")
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