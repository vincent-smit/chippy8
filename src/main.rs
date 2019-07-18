use std::fs::File;
use std::io::prelude::*;
use std::{fmt::Write, num::ParseIntError};

extern crate nom;
use nom::{HexDisplay,IResult,Needed,Offset};




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


    let hex_e = decode_hex("e0").unwrap();

    while pc < length  {
        let first = buffer[pc];
        let second = buffer[pc+1];
        let third = buffer[pc+2];
        let fourth = buffer[pc+3];


        println!("{:x},{:x}",first,second);
        println!("{:x},{:x}",third,fourth);

        match (first, second, third, fourth){
            (0,0,_,0) => {
                println!("CLS")
            }
            (0,0,_,hex_e) => {
                println!("RTS")
            }
            (3,_,_,_) => {
                println!("SKIP.EQ")
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