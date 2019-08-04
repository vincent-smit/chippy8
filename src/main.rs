extern crate nom;
extern crate rand;
extern crate glium;
extern crate image;


#[allow(unused_imports)]
use std::fs::File;
use std::io::prelude::*;
use std::sync::mpsc::channel;
use std::thread::*;
use std::io::Cursor;
use std::sync::mpsc::*;

use glium::{Surface};
use image::{ImageBuffer, RgbImage};

mod cpu;

const M: usize = 256;
const N: usize = 256;


fn main() {
    println!("Hello, world!");

    let filename = r"roms/Fishie.ch8";
    let mut rom_file = File::open(filename).unwrap();
    let mut binary = Vec::new();
    rom_file.read_to_end(&mut binary).unwrap();

    let mut cpu = cpu::CPU::new(binary);
    let (mut eventloop, screen, mut texture) = Screen::new();

    let renderoptions = RenderOptions {linear_interpolation: true};

    let (sender, receiver) = channel();
    let (sender2, receiver2) = sync_channel(1);

    //TODO: Move rom binary to CPU thread
    let cputhread = spawn(move || {
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
                    match opcode & 0xF000 {
                        0xD000 => {
                            println!("Sending GPU instrutions");
                            if sender2.send(gpu_instr).is_err() {
                                break
                            }
                        }
                        _ => {
                            if sender.send(gpu_instr).is_err() {
                            //    break
                            }
                         }
                    }

                }
                None => {
                    println!("No GPU instructions");
                }
            }

            // Store key press state
        }
    });

    // Start Glium eventloop
    let mut stop = false;
    while !stop {
        eventloop.poll_events(|eventloop|  {
            use glium::glutin::{Event, WindowEvent, KeyboardInput};
            use glium::glutin::ElementState::{Pressed, Released};
            use glium::glutin::VirtualKeyCode;


            match eventloop {
                Event::WindowEvent { event, .. } => match event {
                     WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput { state: Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }
                            => stop = true,
                        _ => ()
                        }
                    _ => ()
                    }
                _ => ()
            }
        }
        );

        match receiver2.recv() {
            Ok(data) => recalculate_screen(&screen, &mut texture, &data, &renderoptions),
            Err(..) => break, // Remote end has hung-up
        }
    }

    cputhread.join().unwrap();
}


fn recalculate_screen(display: &glium::Display,
                      texture: &mut glium::texture::texture2d::Texture2d,
                      datavec: &[u8],
                      renderoptions: &RenderOptions)
{

    let interpolation_type = if renderoptions.linear_interpolation {
        glium::uniforms::MagnifySamplerFilter::Linear
    }
    else {
        glium::uniforms::MagnifySamplerFilter::Nearest
    };

    let image = image::load(Cursor::new(&include_bytes!("../roms/opengl.png")[..]),
                            image::PNG).unwrap().to_rgba();
    let image_dimensions = image.dimensions();

    let rawimage2d = glium::texture::RawImage2d {
        data: std::borrow::Cow::Borrowed(datavec),
        width: M as u32,
        height: N as u32,
        format: glium::texture::ClientFormat::U8,
    };

    println!("{:?}", datavec);

    //let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    texture.write(
        glium::Rect {
            left: 0,
            bottom: 0,
            width: M as u32,
            height: N as u32
        },
        rawimage2d);

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




struct Sound {
    hex: u8
}


#[derive(Default)]
struct RenderOptions {
    pub linear_interpolation: bool,
}

struct Screen {
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
}