use std::fs::File;
use std::io;
use std::io::Read;

fn main() {
    use rustdmg::rustdmg::dmg;

    println!("rustdmg");

    println!("Please enter rom file path");
    let mut rom_file_path = String::new();
    io::stdin()
        .read_line(&mut rom_file_path)
        .expect("Failed to read rom file path");
    let rom_file_path = rom_file_path.trim();

    dmg::run(rom_file_path);
}
