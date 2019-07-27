use std::fs::File;
use std::io::prelude::*;
use std::{fmt::Write, num::ParseIntError};
use std::sync::Arc;
use std::cell;

extern crate nom;
use nom::{HexDisplay,IResult,Needed,Offset};




fn main() {
    println!("Hello, world!");

    let filename = r"roms/Fishie.ch8";
    let mut rom_file = File::open(filename).unwrap();
    let mut binary = Vec::new();
    let mut length = rom_file.read_to_end(&mut binary).unwrap();

    let mut count = 100;    //let hex_e = decode_hex("e0").unwrap();
    //let hex_d = decode_hex("d0").unwrap();
    //println!("{:?}", hex_d);
    
    let mut cpu = CPU::new(binary);

    loop {


        //let instr = cpu.fetch_opcode().unwrap();
        cpu.parse_instruction();
        //state.cpu_cycle(opcode);

        count +=1;
        if count >= length {
            break
        }
    }

}

struct CPU {
    memory: Vec<u8>,
    register: Vec<u8>,
    sound_register: u8,
    delay_register: u8,
    pc: usize,
    sp: u8,
    stack: Vec<u8>,
    i: u8,
    //display: Display,
    //sound: Sound,
    //keyboard: Keyboard,
}


impl CPU {

    fn new(binary: Vec<u8>) -> CPU {

    let mut memory: Vec<u8> = Vec::with_capacity(4096);
    let memory = binary.iter().enumerate().map(move|(i,x)| if i == 500 {0} else {*x}).collect();

    println!("{:?}", memory);

    let mut register = Vec::with_capacity(16);
    for nr in 1..16 {
        register.push(nr);
    };
    let mut stack = Vec::with_capacity(16);

    for nr in 1..16 {
        stack.push(nr);
    }


    CPU {
            memory : memory,
            register : register,
            sound_register : 0,
            delay_register : 0,
            pc : 0,
            sp : 0,
            stack : stack,
            i : 0,
        }
    }
    
    fn fetch_opcode<'a>(self: Self) -> Option< Vec<u8>> {
        println!("Fetching opcode..");

        let nibble1 = (self.memory[self.pc] & 0x00F0) >> 4;
        let nibble2 = self.memory[self.pc] & 0x000F; 
        let nibble3 = (self.memory[self.pc+1] & 0x00F0) >> 4;
        let nibble4 = self.memory[self.pc+1] & 0x000F;

        let mut content = Vec::new();
        content.push(nibble1);
        content.push(nibble2);
        content.push(nibble3);
        content.push(nibble4);

        return Some(content)
    }

    fn cpu_cycle(self) {
        // 60times a second, grab an opcode and execute the comment

        // process_opcode()
    }



    fn parse_instruction(&mut self) {

        //let nibble1 = (self.memory[self.pc] & 0x00F0) >> 4;
        //let nibble2 = self.memory[self.pc] & 0x000F; 

        let left: u16 = self.memory[self.pc].clone().into();
        let right: u16 = self.memory[self.pc + 1].clone().into();
        println!("{:x}", left);
        println!("{:x}", right);

        let opcode: u16 = left | right;

        println!("First nibble.. {:x}", opcode & 0xF000);
        println!("Parsing.. {:x}",opcode);

        let mut instr = Vec::new();
        instr.push(1);
        instr.push(2);
        instr.push(3);
        instr.push(4);

        
        match instr[0] {
            0 => {
                match instr[3] {
                    0 => {
                        println!("Clear screen")
                    }
                    _ => {
                        println!("Returns from a subroutine")
                    }
                }
            }

            1 => {
                println!("JUMP NNN");
                let target: u8 = instr.iter().skip(1).sum();
                self.pc = target as usize;
            }

            2 => {
                println!("Calls subroutine at NNN")
            }

            3 => {
                if self.register[instr[1] as usize] == instr.iter().skip(2).sum() {
                    println!("SKIP.EQ");
                    self.pc += 2;
                }
                else {
                    println!("NOT SKIP.EQ")
                }               
            }
            
            4 => {
                if self.register[instr[1] as usize] != instr.iter().skip(2).sum() {
                    println!("SKIP.NE");
                    self.pc += 2;
                }
                else {
                    println!("NOT SKIP.NE")
                }
            }

            5 => {
                if self.register[instr[1] as usize] == self.register[instr[2] as usize] {
                    println!("SKIP.EQ");
                    self.pc += 2;
                }
                else {
                    println!("NOT SKIP.EQ")
                } 
            }
            
            6 => {
                self.register[instr[1] as usize] = instr.iter().skip(2).sum();
                println!("MVI.");
            }

            7 => {
                let count = self.register[instr[1] as usize];
                let count2: u8 =  instr.iter().skip(2).sum();
                self.register[instr[1] as usize] = count + count2;
                println!("Adds NN to VX")
            }

            8 => {
                println!("MOD");
                match instr[3] {
                    0 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[1] as usize] = self.register[instr[2] as usize];
                    }

                    1 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[1] as usize] = self.register[instr[2] as usize] | self.register[instr[2] as usize];
                    }

                    2 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[1] as usize] = self.register[instr[2] as usize] | self.register[instr[2] as usize];
                    }
                    3 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[1] as usize] = self.register[instr[2] as usize] ^ self.register[instr[2] as usize];
                    }
                    4 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[2] as usize] += self.register[instr[2] as usize];
                    }
                    5 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[2] as usize] -= self.register[instr[2] as usize];
                    }
                    6 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[2] as usize] >>= self.register[instr[2] as usize];
                    }
                    7 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[1] as usize] = self.register[instr[2] as usize] ^ self.register[instr[1] as usize];
                    }
                    14 => {
                        println!("Sets VX to the value of VY.");
                        self.register[instr[2] as usize] >>= self.register[instr[2] as usize];
                    } 
                    _ => {
                        println!("Else")
                    }
                }
            }
            
            9 => {
                if self.register[instr[1] as usize] != self.register[instr[2] as usize] {
                    println!("if(Vx!=Vy)");
                    self.pc += 2;
                }
            }
            
            10 => {
                println!("I = NNN");
                let mem_loc: u8 = instr.iter().skip(1).sum();
                println!("{}", mem_loc);
                self.i = self.memory[mem_loc as usize];

            }

            11 => {
                println!("PC=V0+NNN");
                let target: u8 = instr.iter().skip(1).sum();
                let target = target + self.register[0];
                self.pc = target as usize;
            }
            
            12 => {
                println!("Vx=rand()&NN")
            }
            
            13 => {
                println!("draw(Vx,Vy,N)")
            }

            14 => {
                match instr[3] {
                    14 => {
                        println!("if(key()==Vx)")
                    }

                    1 => {
                        println!("if(key()!=Vx)")
                    }
                    _ => {
                        println!("E not implemented!")
                    }
                }
            }

            15 => {
                match (instr[2], instr[3]) {
                    (0,7) => {
                        println!("Vx = get_delay()	")
                    },
                    (0,10) => {
                        println!("Vx = get_key()")
                    },
                    (1,5) => {
                        println!("delay_timer(Vx)")
                    },
                    (1,8) => {
                        println!("sound_timer(Vx)")
                    },
                    (1,14) => {
                        println!("I +=Vx")
                    },
                    (2,9) => {
                        println!("I=sprite_addr[Vx]")
                    },
                    (3,3) => {
                        println!("set_BCD(Vx)")
                    },
                    (5,5) => {
                        println!("reg_dump(Vx,&I)")
                    },
                    (6,5) => {
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
        self.pc += 2;

    }

}

struct Display {
    x: u8,
    y: u8,
}

struct Keyboard {
    hex: u8
}

struct Sound {
    hex: u8
}


pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b);
    }
    s
}