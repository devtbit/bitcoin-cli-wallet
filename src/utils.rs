use std::{
    fs::File,
    path::Path,
    io::{
        Read,
        Write,
    },
    process,
};
use termion::{
    color,
    style,
};

pub fn write_to_file(filename: &str, bytes: &Vec<u8>) {
    let path = Path::new(filename);
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("could not create {}: {}", display, why),
        Ok(file) => file,
    };
    match file.write_all(bytes) {
        Err(why) => panic!("could not write to {}: {}", display, why),
        Ok(_) => (),
    }
}

pub fn read_from_file(filename: &str) -> Vec<u8> {
    let path = Path::new(filename);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("could not open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut s = Vec::new();
    match file.read_to_end(&mut s) {
        Err(why) => panic!("could not read {}: {}", display, why),
        Ok(_) => s,
    }
}

pub fn fatal_kill(message: &str) {
    println!("{}{}Error: {}{}{}{}", color::Fg(color::Red), style::Bold, style::Reset, color::Fg(color::Red), message, style::Reset);
    process::exit(1);
}

pub fn warn(message: &str) {
    println!("{}{}Warning: {}{}{}{}", style::Bold, color::Fg(color::Yellow), style::Reset, color::Fg(color::Yellow), message, style::Reset);
}