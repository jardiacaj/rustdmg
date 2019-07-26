use std::fs::File;
use std::io;
use std::io::Read;

extern crate blit;

fn main() {
    use rustdmg::dmg;

    println!("rustdmg");

    println!("Please enter ROM file path");
    let mut rom_file_path = String::new();
    io::stdin()
        .read_line(&mut rom_file_path)
        .expect("Failed to read rom file path");
    let rom_file_path = rom_file_path.trim();

    let mut dmg = dmg::DMG::new(rom_file_path).unwrap();
    dmg.run();
}
