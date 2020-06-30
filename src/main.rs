use std::env;
use std::io;
use rustdmg::dmg;


fn main() {
    println!("rustdmg");

    let mut args = env::args();
    let mut rom_file_path: String;
    args.next(); // skip first element as it's the called program name
    match args.next() {
        Some(argument) => rom_file_path = argument,
        None => {
            println!("Please enter ROM file path");
            rom_file_path = String::new();
            io::stdin()
                .read_line(&mut rom_file_path)
                .expect("Failed to read rom file path");
            rom_file_path = rom_file_path.trim().to_string();
        }
    }


    let mut dmg = dmg::DMG::new(&rom_file_path).unwrap();
    dmg.run();
}
