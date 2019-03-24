use area8051::{Addr, Isa, Mcu, Mem, Reg};
use std::fs;
use std::sync::Mutex;

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
}

impl Mem for Ec {
    fn load(&self, addr: Addr) -> u8 {
        let mut mcu = self.mcu.lock().unwrap();
        match addr {
            Addr::XRam(i) => {
                xram(&mut mcu, i, None)
            },
            Addr::PMem(i) => if i >= 0x8000 {
                let bank = if mcu.xram[0x1001] & (1 << 7) == 0 {
                    // Use P1[1:0]
                    mcu.load(mcu.p(1)) & 0b11
                } else {
                    // Use ECBB[1:0]
                    mcu.xram[0x1005] & 0b11
                };
                mcu.pmem[(i as usize) + (bank as usize) * 0x8000]
            } else {
                mcu.pmem[i as usize]
            },
            _ => mcu.load(addr),
        }
    }

    fn store(&mut self, addr: Addr, value: u8) {
        let mut mcu = self.mcu.lock().unwrap();
        match addr {
            Addr::XRam(i) => {
                xram(&mut mcu, i, Some(value));
            },
            _ => mcu.store(addr, value),
        }
    }
}

impl Reg for Ec {}

impl Isa for Ec {
    fn pc(&self) -> u16 {
        self.mcu.lock().unwrap().pc()
    }

    fn set_pc(&mut self, value: u16) {
        self.mcu.lock().unwrap().set_pc(value);
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
