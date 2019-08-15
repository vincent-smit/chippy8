extern crate rand;
extern crate glium;
extern crate glutin;

mod cpu;

#[allow(unused_imports)]
use std::fs::File;
use std::thread::*;
use std::sync::mpsc::*;
use std::env;
use std::io::Read;
use std::time;
use glium::{Surface};
use glium::glutin::{Event, WindowEvent, KeyboardInput};
use glium::glutin::ElementState::{Pressed, Released};
use glium::glutin::VirtualKeyCode;
use std::time::Instant;


const WIDTH: usize = 64;
const HEIGHT: usize = 32;


fn main() {
    println!("Hello, CHIP8!");

    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        panic!("Not enough input parameters!")
    }

    let mut file = File::open(&args[1]).expect("Could not open rom file..");
    let fsize = file.metadata().unwrap().len();

    let mut binary = Vec::new();
    file.read_to_end(&mut binary).expect("Could not read binary in memory...");
    drop(file);

    let mut cpu = cpu::CPU::new(binary, fsize);
    //let (mut eventloop, screen, mut texture, mut gl_window) = Screen::new();

    let mut eventsloop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(glium::glutin::dpi::LogicalSize::from((WIDTH as u32, HEIGHT as u32)))
            .with_title("Chippy8 - ".to_owned());
    let context_builder = glutin::ContextBuilder::new();
    let screen = glium::backend::glutin::Display::new(window_builder, context_builder, &eventsloop).unwrap();

    let mut texture = glium::texture::texture2d::Texture2d::empty_with_format(
                &screen,
                glium::texture::UncompressedFloatFormat::U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                WIDTH as u32,
                HEIGHT as u32)
            .unwrap();


    let renderoptions = RenderOptions {linear_interpolation: true};

    let (sender, receiver) = channel();
    let (sender2, receiver2) = sync_channel(1);

    //TODO: Move rom binary to CPU thread
    let cputhread = spawn(move || {
        loop {
            //println!("Start loooooooop");
            let opcode = cpu.fetch_opcode().unwrap();
            cpu.parse_instruction(opcode);
            if cpu.draw_flag == true {
                if sender2.send(cpu.pixels.clone()).is_err() {
                    panic!("Could not send pixel data")
                }
            }

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

            // Store key press state
            cpu.draw_flag = false;


            sleep(time::Duration::from_millis(16));

            'recv: loop {
                match receiver.try_recv() {
                    Ok(event) => {
                        match event {
                            0x1 => println!("Hey"),
                            0x2 => println!("Cowboy"),
                            _ =>   println!("Ola") 
                        }
                    },
                    Err(TryRecvError::Empty) => break 'recv,
                    Err(TryRecvError::Disconnected) => panic!("Howloa"),
                }
            }
        }
    });

    // Start Glium eventloop
    let mut stop = false;
    while !stop {
        eventsloop.poll_events(|ev|  {
            match ev {
                Event::WindowEvent { event, .. } => 
                    match event {
                    WindowEvent::CloseRequested => { stop = true },
                    WindowEvent::KeyboardInput { input, .. } => {
                        match input {
                            KeyboardInput { state: Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }
                                => { stop = true },

                            KeyboardInput { state: Pressed, virtual_keycode: Some(vkey), .. } => {
                                match vkey {
                                VirtualKeyCode::Key1 => sender.send(0x1).unwrap(),
                                VirtualKeyCode::Key2 => sender.send(0x2).unwrap(),
                                VirtualKeyCode::Key3 => sender.send(0x3).unwrap(),
                                VirtualKeyCode::Q    => println!("Hello"),
                                _ => ()
                                }
                            },

                            KeyboardInput { state: Released, virtual_keycode: Some(vkey), .. } => {
                                match vkey {
                                VirtualKeyCode::Key1 => sender.send(0x1).unwrap(),
                                VirtualKeyCode::Key2 => sender.send(0x2).unwrap(),
                                VirtualKeyCode::Key3 => sender.send(0x3).unwrap(),
                                VirtualKeyCode::Q    => sender.send(0x4).unwrap(),
                                _ => ()
                                }
                            },
                            _ => ()

                            }
                        }
                    _ => ()
                }
                _ => ()
            }
        });

        match receiver2.try_recv() {
            Ok(data) => recalculate_screen(&screen, &mut texture, &data, &renderoptions),
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Other thread disconnected!"),
        }
        //sleep(time::Duration::from_millis(16));

    }
    cputhread.join().unwrap();
    drop(eventsloop);
}




fn recalculate_screen(display: &glium::Display,
                      texture: &mut glium::texture::texture2d::Texture2d,
                      datavec: &Vec<(u8,u8,u8)>,
                      renderoptions: &RenderOptions)
{

    let interpolation_type = if renderoptions.linear_interpolation {
        glium::uniforms::MagnifySamplerFilter::Linear
    }
    else {
        glium::uniforms::MagnifySamplerFilter::Nearest
    };

    let rawimage2d = glium::texture::RawImage2d {
        data: std::borrow::Cow::Borrowed(datavec),
        width: WIDTH as u32,
        height: HEIGHT as u32,
        format: glium::texture::ClientFormat::U8U8U8
    };

    //println!("{:?}", datavec);

    texture.write(
        glium::Rect {
            left: 0,
            bottom: 0,
            width: WIDTH as u32,
            height: HEIGHT as u32
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



#[derive(Default)]
struct RenderOptions {
    pub linear_interpolation: bool,
}

struct Screen {
}

impl Screen {

    fn new() -> (glium::glutin::EventsLoop, glium::backend::glutin::Display, glium::texture::texture2d::Texture2d) {
        let eventsloop = glium::glutin::EventsLoop::new();
        let window_builder = glium::glutin::WindowBuilder::new()
            .with_dimensions(glium::glutin::dpi::LogicalSize::from((WIDTH as u32, HEIGHT as u32)))
            .with_title("Chippy8 - ".to_owned());
        let context_builder = glium::glutin::ContextBuilder::new();
        let display = glium::backend::glutin::Display::new(window_builder, context_builder, &eventsloop).unwrap();

        let texture = glium::texture::texture2d::Texture2d::empty_with_format(
                &display,
                glium::texture::UncompressedFloatFormat::U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                WIDTH as u32,
                HEIGHT as u32)
            .unwrap();
        (eventsloop, display, texture)
    }
}