use std::env;
use std::io;
use rustdmg::dmg;


fn main() {
    println!("rustdmg");

    let mut args = env::args();
    let mut rom_file_path: Option<String> = None;
    let mut debug = false;
    args.next(); // skip first element as it's the called program name
    while let Some(argument) = args.next() {
        if argument == "--debug" {
            debug = true;
        } else {
            rom_file_path = Some(argument);
        }
    }

    let mut dmg = dmg::DMG::new(&rom_file_path.unwrap()).unwrap();
    dmg.cpu.debug = false;
    dmg.run();
}
