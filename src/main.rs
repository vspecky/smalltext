mod huffman;

use huffman::{
    encoder::Encoder,
    decoder::Decoder,
};

use std::{
    vec::Vec,
    env,
};

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        return Err("Invalid number of arguments.");
    }

    if args[1] != "-c" && args[1] != "-d" {
        return Err("Usage: smalltext (-c | -d) path/to/file");
    }

    if args[1] == "-c" {
        let mut enc = match Encoder::new(args[2].clone()) {
            Ok(e) => e,
            Err(msg) => return Err(msg),
        };
    
        match enc.compress() {
            Err(_) => return Err("Error compressing file"),
            Ok(_) => Ok(())
        }
    } else {
        let mut dec = match Decoder::new(args[2].clone()) {
            Ok(d) => d,
            Err(_) => return Err("The file does not exist"),
        };

        match dec.decompress() {
            Err(msg) => return Err(msg),
            Ok(_) => Ok(())
        }
    }
}
