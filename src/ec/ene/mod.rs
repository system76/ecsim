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
}

impl Mem for Ec {
    fn load(&self, addr: Addr) -> u8 {
        match addr {
            Addr::XRam(i) => {
                xram(self, i, None)
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
    }
}
