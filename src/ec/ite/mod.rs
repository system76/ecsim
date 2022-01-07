// SPDX-License-Identifier: MIT

use area8051::{Addr, Isa, Mcu, Mem, Reg};
use std::sync::Mutex;

use crate::Spi;
use self::xram::xram;

mod xram;

pub struct Ec {
    pub id: u16,
    pub version: u8,
    pub mcu: Mutex<Mcu>,
    pub spi: Mutex<Spi>,
    pub xmem: Mutex<Box<[u8]>>,
    pub superio_addr: u8,
    pub steps: u64,
}

impl Ec {
    pub fn new(id: u16, version: u8, pmem: Box<[u8]>, xmem: Box<[u8]>) -> Self {
        Self {
            id,
            version,
            mcu: Mutex::new(Mcu::new(pmem)),
            spi: Mutex::new(Spi::new()),
            xmem: Mutex::new(xmem),
            superio_addr: 0,
            steps: 0,
        }
    }

    pub fn scar(&self) -> &'static [(usize, usize, usize)] {
        match self.id {
            0x5570 => &[
                (0x1040, 0x0000, 4096),
            ],
            0x8587 => &[
                (0x1040, 0x0000, 2048),
                (0x1043, 0x0800, 1024),
                (0x1046, 0x0C00, 512),
                (0x1049, 0x0E00, 256),
                (0x104C, 0x0F00, 256)
            ],
            _ => panic!("Ec::scar not implemented for {:04X}", self.id)
        }
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

                for &(reg, base, size) in self.scar() {
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
        for (reg, _, _) in self.scar() {
            mcu.xram[reg + 2] = 0b11;
        }

        // Set default INTC IVECT
        mcu.xram[0x1110] = 0x10;

        // Set CHIP ID
        mcu.xram[0x2000] = (self.id >> 8) as u8;
        mcu.xram[0x2001] = self.id as u8;
        mcu.xram[0x2002] = self.version;
    }
}
