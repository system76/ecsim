// SPDX-License-Identifier: MIT

use area8051::{Addr, Isa, Mcu, Mem, Reg};
use std::sync::Mutex;

use crate::{Spi, xram};

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

        // SMFI
        mcu.xram[0x1001] = if self.id == 0x5570 {
            0b0011_1111
        } else {
            0b1011_1111
        };
        mcu.xram[0x1020] = 0b0000_1000;
        mcu.xram[0x1032] = 0b0000_0011;
        mcu.xram[0x1036] = 0b1000_0000;

        // Disable SCAR
        for (reg, _, _) in self.scar() {
            mcu.xram[reg + 2] = if self.id == 0x5570 {
                0b111
            } else {
                0b11
            };
        }

        // INTC
        mcu.xram[0x1110] = 0x10;

        // KBC
        mcu.xram[0x1202] = 0b0000_0111;

        // PMC
        mcu.xram[0x1506] = 0b0100_0000;
        mcu.xram[0x1516] = 0b0100_0000;

        // GPIO
        mcu.xram[0x1600] = 0b0000_0100;
        mcu.xram[0x1607] = 0b0000_0001;
        if self.id == 0x5570 {
            mcu.xram[0x16E5] = 0b0000_0110;
        }
        mcu.xram[0x16F2] = 0b0100_0000;
        mcu.xram[0x16F5] = 0b0000_1111;

        // PS/2
        mcu.xram[0x1700] = 0b0000_0001;
        mcu.xram[0x1701] = 0b0000_0001;
        mcu.xram[0x1702] = 0b0000_0001;

        // PWM
        mcu.xram[0x1801] = 0xFF;
        mcu.xram[0x180D] = 0b0101_0101;
        mcu.xram[0x1843] = 0xFF;

        // ADC
        mcu.xram[0x1900] = 0b1000_0000;
        mcu.xram[0x1901] = 0b1000_0000;
        mcu.xram[0x1904] = 0b0001_1111;
        mcu.xram[0x1906] = 0b0001_1111;
        mcu.xram[0x1909] = 0b0001_1111;
        mcu.xram[0x190C] = 0b0001_1111;

        // DAC
        mcu.xram[0x1A00] = 0b0001_0000;
        mcu.xram[0x1A01] = 0b0011_1100;

        // SMBus
        if self.id == 0x5570 {
            mcu.xram[0x1C26] = 0x19;
        }
        mcu.xram[0x1C34] = 0b0000_0100;
        if self.id == 0x5570 {
            mcu.xram[0x1C40] = 0b0000_0100;
            mcu.xram[0x1C41] = 0b0000_0100;
            mcu.xram[0x1CA9] = 0b0000_1100;
        }

        // KBC
        mcu.xram[0x1D22] = 0b0000_0001;

        // ECPM
        mcu.xram[0x1E03] = 0b0000_0001;
        mcu.xram[0x1E04] = 0b0111_0000;
        mcu.xram[0x1E05] = 0b0100_0001;
        mcu.xram[0x1E06] = 0b0000_0001;
        mcu.xram[0x1E09] = 0b0000_0001;

        // GCTRL
        mcu.xram[0x2000] = (self.id >> 8) as u8;
        mcu.xram[0x2001] = self.id as u8;
        mcu.xram[0x2002] = self.version;
        mcu.xram[0x2006] = if self.id == 0x5570 {
            0b01001100
        } else {
            0b10001100
        };

        if self.id == 0x5570 {
            // eSPI slave
            mcu.xram[0x3104] = 0b0000_0011;
            mcu.xram[0x3105] = 0b0000_0010;
            mcu.xram[0x3107] = 0b0000_1111;
            mcu.xram[0x310A] = 0b0001_0001;
            mcu.xram[0x310E] = 0b0000_0111;
            mcu.xram[0x3112] = 0b0000_0001;
            mcu.xram[0x3113] = 0b0001_0000;
            mcu.xram[0x3116] = 0b0001_0001;
            mcu.xram[0x3117] = 0b0010_0100;
            mcu.xram[0x311A] = 0b0000_0100;
            mcu.xram[0x311B] = 0b0000_0001;

            // eSPI VW
            mcu.xram[0x3200] = 0b0000_0011;
            mcu.xram[0x3202] = 0b0000_0011;
            mcu.xram[0x3203] = 0b0000_0011;
            mcu.xram[0x3204] = 0b0000_0011;
            mcu.xram[0x3205] = 0b0000_0011;
            mcu.xram[0x3206] = 0b0000_0011;
            mcu.xram[0x3207] = 0b0000_0011;
            mcu.xram[0x3240] = 0b0000_0011;
            mcu.xram[0x3241] = 0b0000_0011;
            mcu.xram[0x3242] = 0b0000_0011;
            mcu.xram[0x3243] = 0b0000_0011;
            mcu.xram[0x3244] = 0b0000_0011;
            mcu.xram[0x3245] = 0b0000_0011;
            mcu.xram[0x3246] = 0b0000_0011;
            mcu.xram[0x3247] = 0b0000_0011;
        }
    }
}
