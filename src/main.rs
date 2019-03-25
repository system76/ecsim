use area8051::{Addr, Isa, Mcu, Mem, Reg};
use std::fs;
use std::sync::Mutex;

use self::spi::Spi;
mod spi;

use self::xram::xram;
mod xram;

pub struct Ec {
    mcu: Mutex<Mcu>,
    spi: Mutex<Spi>,
    xmem: Mutex<Box<[u8]>>,
}

impl Ec {
    pub fn new(pmem: Box<[u8]>) -> Self {
        Self {
            mcu: Mutex::new(Mcu::new(pmem.clone())),
            spi: Mutex::new(Spi::new()),
            xmem: Mutex::new(pmem),
        }
    }

    pub fn scar() -> &'static [(usize, usize, usize)] {
        &[
            (0x1040, 0x0000, 2048),
            (0x1043, 0x0800, 1024),
            (0x1046, 0x0C00, 512),
            (0x1049, 0x0E00, 256),
            (0x104C, 0x0F00, 256)
        ]
    }
}

impl Mem for Ec {
    fn load(&self, addr: Addr) -> u8 {
        match addr {
            Addr::XRam(i) => {
                xram(self, i, None)
            },
            Addr::PMem(i) => {
                let mcu = self.mcu.lock().unwrap();

                let real = if i >= 0x8000 {
                    let bank = if mcu.xram[0x1001] & (1 << 7) == 0 {
                        // Use P1[1:0]
                        mcu.load(mcu.p(1)) & 0b11
                    } else {
                        // Use ECBB[1:0]
                        mcu.xram[0x1005] & 0b11
                    };
                    (i as usize) + (bank as usize) * 0x8000
                } else {
                    i as usize
                };

                for &(reg, base, size) in Self::scar() {
                    let l = mcu.xram[reg];
                    let m = mcu.xram[reg + 1];
                    let h = mcu.xram[reg + 2];

                    let value = {
                        (l as usize) |
                        (m as usize) << 8 |
                        ((h as usize) & 0b11) << 16
                    };

                    if real >= value && real < value + size {
                        return mcu.xram[(real - value) + base];
                    }
                }

                mcu.pmem[real]
            },
            _ => {
                let mcu = self.mcu.lock().unwrap();
                mcu.load(addr)
            },
        }
    }

    fn store(&mut self, addr: Addr, value: u8) {
        match addr {
            Addr::XRam(i) => {
                xram(self, i, Some(value));
            },
            _ => {
                let mut mcu = self.mcu.lock().unwrap();
                mcu.store(addr, value);
            },
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

    fn reset(&mut self) {
        let mut mcu = self.mcu.lock().unwrap();

        mcu.reset();

        // Disable SCAR
        for (reg, _, _) in Self::scar() {
            mcu.xram[reg + 2] = 0b11;
        }
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
