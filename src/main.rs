use std::fs::File;
use std::io::prelude::*;


fn main() {
    println!("Hello, world!");

    let mut memory: Vec<u8> = Vec::with_capacity(4096);

    let mut register: Vec<u8> = Vec::with_capacity(16);

    let mut sound_register: u8 = 0;

    let mut delay_register: u8 = 0;

    let mut pc: usize = 0;

    let mut stack_pointer: u8 = 0;

    let mut stack: Vec<u16> = Vec::with_capacity(16);

    let filename = r"roms/Fishie.ch8";

    let mut file = File::open(filename).unwrap();
    //let reader = BufReader::new(file);

    // read into a String, so that you don't need to do the conversion.
    let mut buffer = Vec::new();
    let length = file.read_to_end(&mut buffer).unwrap();

    println!("{:?}", buffer);

    while pc < length  {
        let first = (buffer[pc] & 0x00F0) >> 4;
        let second = buffer[pc] & 0x000F;

        println!("{},{}",first,second);
        println!("{}",second);


        match (first, second){
            (0x0,0x0) => {
                println!("Clear")
            }
            _ => {
                println!("Everything else")
            }
            

        }

        pc +=2
    }



    //for line in reader.lines() {
    //    let bytes = line.unwrap();
    //    println!("{}",bytes);
    //};


}


struct Display {
    x: u8,
    y: u8,
}

struct Keyboard {
    hex: u8
}