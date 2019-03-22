use area8051::{Mcu, Rom, Ram};
use std::fs;

use self::iram::IRam;
mod iram;

use self::xram::XRam;
mod xram;

fn main() {
    let mut pmem = fs::read("ec.rom").expect("failed to read ec.rom");
    let mut iram = IRam::new();
    let mut xram = XRam::new();

    let mut mcu = Mcu::new(0, pmem, iram, xram);

    loop {
        mcu.step();

        if mcu.pc == 0 {
            eprintln!("reset!");
            break;
        }
    }
}
