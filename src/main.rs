use area8051::{Addr, Isa, Mcu, Mem, Reg};
use std::fs;
use std::sync::{Mutex, MutexGuard};

use self::xram::xram;
mod xram;

pub struct Ec {
    mcu: Mutex<Mcu>,
}

impl Ec {
    pub fn new(pmem: Box<[u8]>) -> Self {
        Self {
            mcu: Mutex::new(Mcu::new(pmem))
        }
    }

    fn mcu(&self) -> MutexGuard<Mcu> {
        self.mcu.lock().unwrap()
    }
}

impl Mem for Ec {
    fn load(&self, addr: Addr) -> u8 {
        match addr {
            Addr::XRam(i) => {
                xram(&mut self.mcu(), i, None)
            },
            _ => self.mcu().load(addr),
        }
    }

    fn store(&mut self, addr: Addr, value: u8) {
        match addr {
            Addr::XRam(i) => {
                xram(&mut self.mcu(), i, Some(value));
            },
            _ => self.mcu().store(addr, value),
        }
    }
}

impl Reg for Ec {}

impl Isa for Ec {
    fn pc(&self) -> u16 {
        self.mcu().pc()
    }

    fn set_pc(&mut self, value: u16) {
        self.mcu().set_pc(value);
    }
}

fn main() {
    let pmem = fs::read("ec.rom").expect("failed to read ec.rom");

    let mut ec = Ec::new(pmem.into_boxed_slice());

    ec.reset();

    loop {
        ec.step();

        if ec.pc() == 0 {
            eprintln!("reset!");
            break;
        }
    }
}
