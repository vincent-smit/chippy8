#[macro_use]
extern crate nom;
extern crate rand;
extern crate glium;
extern crate image;


#[allow(unused_imports)]


use std::fs::File;
use std::io::prelude::*;
use std::{fmt::Write, num::ParseIntError};
use std::sync::mpsc::channel;
use std::thread::*;
use std::error::Error;
use std::io::Cursor;
use std::sync::mpsc;

use rand::Rng;
use glium::{glutin, Surface, Display};
use glium::index::PrimitiveType;
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};


const M: usize = 256;
const N: usize = 256;


fn main() {
    println!("Hello, world!");

    let filename = r"roms/Fishie.ch8";
    let mut rom_file = File::open(filename).unwrap();
    let mut binary = Vec::new();
    let length = rom_file.read_to_end(&mut binary).unwrap();

    let mut cpu = CPU::new(binary);
    let (mut eventloop, mut screen, mut texture) = Screen::new();

    let mut renderoptions = RenderOptions {linear_interpolation: true};

    let img: RgbImage = ImageBuffer::new(64, 32);

    //img.put_pixel(32, 16, pixel);
    //img.put_pixel(33, 17, pixel);
    

    let (sender, receiver) = mpsc::channel();
    let (sender2, receiver2) = mpsc::sync_channel(1);

    //TODO: Move rom binary to CPU thread
    let cputhread = spawn(move || {
        let mut count = 100;
        
        loop {

            // return result, if result we pass vec[u8] as data to refresh the screen by passing the instruction to main thread.
            let opcode = cpu.fetch_opcode().unwrap();

            let decode_execute = cpu.parse_instruction(opcode);

            // Update timers
            if cpu.delay_register > 0 {
                cpu.delay_register -=  1;
            }
            
            if cpu.sound_register > 0 {
                if cpu.sound_register == 1 {
                  println!("BEEP!");
                }
                cpu.sound_register -= 1;
            }  

            match decode_execute {
                Some(gpu_instr) => {
                    if sender2.send(gpu_instr).is_err() {
                        break
                    }
                }
                None => {
                    println!("No GPU instructions");
                }
            }

            // Store key press state

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
                _ => ()
            }
        });

        match receiver2.recv() {
            Ok(data) => recalculate_screen(&screen, &mut texture, &renderoptions),
            Err(..) => break, // Remote end has hung-up
        }
    }

    cputhread.join().unwrap();
}


fn recalculate_screen(display: &glium::Display,
                      texture: &mut glium::texture::texture2d::Texture2d,
                      //datavec: &[u8],
                      renderoptions: &RenderOptions)
{
    use glium::Surface;

    let interpolation_type = if renderoptions.linear_interpolation {
        glium::uniforms::MagnifySamplerFilter::Linear
    }
    else {
        glium::uniforms::MagnifySamplerFilter::Nearest
    };

    let image = image::load(Cursor::new(&include_bytes!("../roms/opengl.png")[..]),
                            image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();

    //let rawimage2d = glium::texture::RawImage2d {
    //    data: std::borrow::Cow::Borrowed(datavec),
    //    width: M as u32,
    //    height: N as u32,
    //    format: glium::texture::ClientFormat::U8U8U8,
    //};

    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);


    texture.write(
        glium::Rect {
            left: 0,
            bottom: 0,
            width: M as u32,
            height: N as u32
        },
        image);

    // We use a custom BlitTarget to transform OpenGL coordinates to row-column coordinates
    let target = display.draw();
    let (target_w, target_h) = target.get_dimensions();
    texture.as_surface().blit_whole_color_to(
        &target,
        &glium::BlitTarget {
            left: 0,
            bottom: target_h,
            width: target_w as i32,
            height: -(target_h as i32)
        },
        interpolation_type);
    target.finish().unwrap();
}



struct Keyboard {
    key: Vec<u8>
}

struct Sound {
    hex: u8
}

struct Screen {
    y: u8,
    x: u8
}

#[derive(Default)]
struct RenderOptions {
    pub linear_interpolation: bool,
}



impl Screen {

    fn new() -> (glium::glutin::EventsLoop, glium::backend::glutin::Display, glium::texture::texture2d::Texture2d) {
        let mut eventsloop = glium::glutin::EventsLoop::new();
        let window_builder = glium::glutin::WindowBuilder::new()
            .with_dimensions(glium::glutin::dpi::LogicalSize::from((M as u32, N as u32)))
            .with_title("Chippy8 - ".to_owned());
        let context_builder = glium::glutin::ContextBuilder::new();
        let display = glium::backend::glutin::Display::new(window_builder, context_builder, &eventsloop).unwrap();

        let mut texture = glium::texture::texture2d::Texture2d::empty_with_format(
                &display,
                glium::texture::UncompressedFloatFormat::U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                M as u32,
                N as u32)
            .unwrap();
        (eventsloop, display, texture)
    }

    fn refresh(&mut self,
        texture: &mut glium::texture::texture2d::Texture2d,
        datavec: &[u8],
        renderoptions: &RenderOptions
        ) {
        println!("Hello");
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

        //let mut grid = vec![0; N * M];

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
    
    fn fetch_opcode(&mut self) -> Result<u16> {
        let left: u16 = self.memory[self.pc].clone().into();
        let right: u16 = self.memory[self.pc + 1].clone().into();

        let opcode: u16 = (left << 8| right) as u16;

        println!("Parsing.. {:x} (in hex) / {:b} (in binary)",opcode, opcode);
        println!("First nibble '{:b}', second nibble '{:b}', third nibble '{:b}', fourth nibble '{:b}'", opcode & 0xFFFF,opcode & 0x0FFF, opcode & 0x00FF, opcode & 0x0F );
        println!("First nibble '{:x}', second nibble '{:x}', third nibble '{:x}', fourth nibble '{:x}'", opcode & 0xFFFF,opcode & 0x0FFF, opcode & 0x00FF, opcode & 0x0F );

        Ok(opcode)
    }

    fn parse_instruction(&mut self, opcode: u16) -> Option<Vec<u8>> {

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
                self.stack[self.sp] = self.pc;
                self.sp += 1;
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
                let height = opcode & 0x000F;

                //for (i, row) in self.grid.iter().enumerate() {
                //    for (j, col) in row.iter().enumerate() {
                //        print!("{}", col);
                //    }
                //    println!()
                //}


                let mut texture: Vec<u8> = Vec::new(); 
                for i in 1..height {
                    texture.push(self.register[i as usize]);
                }

                self.pc += 2;

                let result: Vec<u8> = Vec::new();
                Some(result);
            }

            // TODO: Keys
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        println!("if(key()==Vx)")
                        if self.key[(opcode & 0x0F00) >> 8] != 0 {
                            self.pc += 4;
                        }
                        else {
                            self.pc += 2;
                        }
                    }

                    0x00A1 => {
                        println!("if(key()!=Vx)")
                        if self.key[(opcode & 0x0F00) >> 8] != 1 {
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
                        self.i += self.register[(opcode & 0x0F00 >> 8) as usize];
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
