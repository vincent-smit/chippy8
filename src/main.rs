#[macro_use]
extern crate nom;
extern crate rand;
extern crate glium;

#[allow(unused_imports)]


use std::fs::File;
use std::io::prelude::*;
use std::{fmt::Write, num::ParseIntError};
use std::sync::mpsc::channel;
use std::thread::*;

use rand::Rng;
use glium::{glutin, Surface, Display};
use glium::index::PrimitiveType;



fn main() {
    println!("Hello, world!");

    let filename = r"roms/Fishie.ch8";
    let mut rom_file = File::open(filename).unwrap();
    let mut binary = Vec::new();
    let length = rom_file.read_to_end(&mut binary).unwrap();

    let mut cpu = CPU::new(binary);
    let (mut eventloop, mut screen) = Screen::new();

    let (sender, receiver) = channel();

    //TODO: Move rom binary to thread
    let cputhread = spawn(move || {
        let mut count = 100;
        loop {

            // return result, if result we pass vec[u8] as data to refresh the screen by passing the instruction to main thread.
            match cpu.parse_instruction() {
                Some(instr) => {
                    if sender.send(instr).is_err() {
                    break
                    }
                }
                None => {
                    println!("No GPU instructions");
                }
            }

            count +=1;
            if count >= length {
                break
            }
        }
    });

    // Start Glium eventloop

    let mut closed = false;
    while !closed {
        eventloop.poll_events(|eventloop|  {
            match eventloop {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::CloseRequested => closed = true,
                    _ => (),
                },
                _ => (),
            }
        });
    }

    cputhread.join();
}


struct Keyboard {
    hex: u8
}

struct Sound {
    hex: u8
}

struct Screen {
    x: u8,
    y: u8,
}

#[derive(Default)]
struct RenderOptions {
    pub linear_interpolation: bool,
}



impl Screen {

    fn new() -> (glium::glutin::EventsLoop, glium::backend::glutin::Display) {
        let mut eventsloop = glium::glutin::EventsLoop::new();
        let window_builder = glium::glutin::WindowBuilder::new()
            .with_dimensions(glium::glutin::dpi::LogicalSize::from((64, 32)))
            .with_title("Chippy8 - ".to_owned());
        let context_builder = glium::glutin::ContextBuilder::new();
        let display = glium::backend::glutin::Display::new(window_builder, context_builder, &eventsloop).unwrap();

        let mut texture = glium::texture::texture2d::Texture2d::empty_with_format(
                &display,
                glium::texture::UncompressedFloatFormat::U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                64 as u32,
                32 as u32)
            .unwrap();
        (eventsloop, display)
    }

    fn refresh(&mut self,
        texture: &mut glium::texture::texture2d::Texture2d,
        datavec: &[u8],
        renderoptions: &RenderOptions
        ) {
        println!("Hello");;
    }
}


struct CPU {
    memory: Vec<u8>,
    register: Vec<u8>,
    sound_register: u8,
    delay_register: u8,
    pc: usize,
    sp: usize,
    stack: Vec<usize>,
    i: u8,
}


impl CPU {

    fn new(binary: Vec<u8>) -> CPU {

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
                pc : 0x200,
                sp : 0,
                stack : stack,
                i : 0,
            }
    }
    
    fn fetch_opcode<'a>(self: Self) -> Option<Vec<u8>> {
        println!("Fetching opcode..");
        let mut content = Vec::new();
        return Some(content)
    }

    fn cpu_cycle(self) {
        // 60times a second, grab an opcode and execute the comment
        // process_opcode()
    }



    fn parse_instruction(&mut self) -> Option<Vec<u8>> {

        let left: u16 = self.memory[self.pc].clone().into();
        let right: u16 = self.memory[self.pc + 1].clone().into();

        //println!("{:b}", left);
        //println!("{:b}", right);
        //println!("{:x}", left);
        //println!("{:x}", right);

        let opcode: u16 = (left << 8| right) as u16;

        println!("Parsing.. {:x}",opcode);
        println!("Parsing.. {:b}",opcode);
        println!("First nibble '{:b}', second nibble '{:b}', third nibble '{:b}', fourth nibble '{:b}'", opcode & 0xFFFF,opcode & 0x0FFF, opcode & 0x00FF, opcode & 0x0F );
        println!("First nibble '{:x}', second nibble '{:x}', third nibble '{:x}', fourth nibble '{:x}'", opcode & 0xFFFF,opcode & 0x0FFF, opcode & 0x00FF, opcode & 0x0F );

        println!("{}", opcode & 0xF000);
        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0xF {
                    0 => {
                        println!("Clear screen");
                        self.pc += 2;
                    }
                    _ => {
                        println!("Returns from a subroutine");
                        self.pc =  self.stack[self.sp]
                    }
                }
            }

            0x1000 => {
                println!("JUMP NNN");
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                let target = opcode & 0xFFF;
                self.pc = target as usize;
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
                    self.pc += 2;
                }
                else {
                    println!("NOT SKIP.NE")
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
                    4 => {
                        println!("Vx += Vy");
                        if self.register[(opcode & 0x00F0) as usize  >> 4] > (0xFF - self.register[(opcode & 0x0F00) as usize >> 8]) {
                            self.register[0xF] = 1; //carry
                        }
                        else {
                            self.register[0xF] = 0;
                            self.register[(opcode & 0x0F00) as usize >> 8] += self.register[(opcode & 0x00F0) as usize >> 4];
                        }
                    }
                    5 => {
                        println!("Vx += Vy");
                        let borrow = self.register[(opcode & 0x0F00) as usize] - self.register[(opcode & 0x00F0) as usize];
                        if borrow > 0 {
                            self.register[0xF] = 0; //carry
                        }
                        else {
                            self.register[0xF] = 1;
                        }
                    }
                    6 => {
                        println!("{:b}",opcode & 0x0F00 >> 1);
                        println!("Vx>>=1");
                        self.register[0xF] = self.register[(opcode & 0x0F00) as usize]  >>1
                    }
                    7 => {
                        println!("Vx=Vy-Vx");
                        let borrow = self.register[(opcode & 0x0F00) as usize] - self.register[(opcode & 0x00F0) as usize];
                        if borrow > 0 {
                            self.register[0xF] = 0; //carry
                        }
                        else {
                            self.register[0xF] = 1;
                        }
                    }
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
            
            0xD000 => {

                let x = self.register[((opcode & 0x0F00) >> 8) as usize];
                let y = self.register[((opcode & 0x00F0) >> 4) as usize];
                let height = (opcode & 0x000F);

                self.register[0xF] = 0;
                println!("draw(Vx,Vy,N)");
                self.pc += 2;

                let result: Vec<u8> = Vec::new();
                Some(result);
            }

            0xE000 => {
                self.pc += 2;
                match opcode & 0xF {
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

            0xF000 => {
                self.pc += 2;
                match (opcode & 0xFF, opcode & 0xF) {
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
        None

    }

}
