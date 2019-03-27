use area8051::{Addr, Isa, Mem};
use std::fs;

pub use self::ec::Ec;
mod ec;

pub use self::spi::Spi;
mod spi;

pub use self::xram::xram;
mod xram;

fn main() {
    let pmem = fs::read("ec.rom").expect("failed to read ec.rom");

    let mut ec = Ec::new(pmem.into_boxed_slice());

    ec.reset();

    loop {
        ec.step();

        // Check pcon for idle or power down
        let pcon = ec.load(Addr::Reg(0x87));
        if (pcon & 0b11) != 0 {
            panic!("unimplemented PCON 0x{:02X}", pcon);
        }

        if ec.pc() == 0 {
            eprintln!("reset!");
            break;
        }
    }
}
