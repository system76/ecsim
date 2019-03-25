use area8051::{Addr, Mem};

use crate::Ec;

#[cfg(feature = "debug_xram")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_xram"))]
macro_rules! debug {
    ($($arg:tt)*) => (());
}

pub fn xram(ec: &Ec, address: u16, new_opt: Option<u8>) -> u8 {
    let mut mcu = ec.mcu.lock().unwrap();
    let mut spi = ec.spi.lock().unwrap();
    let mut xmem = ec.xmem.lock().unwrap();

    debug!("\n[xram 0x{:04X}", address);

    let mut old = mcu.load(Addr::XRam(address));

    match address {
        // Scratch SRAM
        0x0000 ... 0x0FFF => {
            debug!(" (SRAM)");
        },
        // SMFI
        0x1000 ... 0x107F => {
            let base = 0x1000;
            let offset = address - base;
            debug!(" (SMFI 0x{:02X}", offset);

            let mut scar_dma = |scar| {
                let (reg, base, size) = Ec::scar()[scar];

                let l = mcu.xram[reg];
                let m = mcu.xram[reg + 1];
                let h = mcu.xram[reg + 2];

                let value = {
                    (l as usize) |
                    (m as usize) << 8 |
                    ((h as usize) & 0b11) << 16
                };

                debug!(" [SCAR{} DMA 0x{:04X} = 0x{:04X}]", scar, base, value);
                for i in 0..size {
                    mcu.xram[base + i] = mcu.pmem[value + i];
                }
            };

            match offset {
                0x00 => debug!(" FBCFG"),
                0x01 => debug!(" FPCFG"),
                0x07 => debug!(" UNKNOWN"),
                0x20 => debug!(" SMECCS"),
                0x36 => debug!(" HCTRL2R"),
                0x3B => debug!(" ECINDAR0"),
                0x3C => debug!(" ECINDAR1"),
                0x3D => debug!(" ECINDAR2"),
                0x3E => debug!(" ECINDAR3"),
                0x3F => {
                    debug!(" ECINDDR");

                    let a0 = mcu.load(Addr::XRam(base + 0x3B));
                    let a1 = mcu.load(Addr::XRam(base + 0x3C));
                    let a2 = mcu.load(Addr::XRam(base + 0x3D));
                    let a3 = mcu.load(Addr::XRam(base + 0x3E));
                    let a = {
                        (a0 as usize) |
                        (a1 as usize) << 8 |
                        (a2 as usize) << 16 |
                        (a3 as usize) << 24
                    };

                    debug!(" [flash address 0x{:08X}", a);
                    let flash = match (a3 >> 6) & 0b11 {
                        0b00 | 0b11 => {
                            debug!(" (external)");
                            &mut xmem
                        },
                        0b01 => {
                            debug!(" (internal)");
                            &mut mcu.pmem
                        },
                        unknown => {
                            panic!("unknown ECIND flash chip 0b{:02b}", unknown);
                        }
                    };
                    debug!("]");

                    if a3 & 0xF == 0xF {
                        match a1 {
                            0xFD => {
                                // Enable chip, send or receive
                                debug!(" [follow enable]");
                                if let Some(new) = new_opt {
                                    spi.input.push_back(new);
                                } else {
                                    spi.step(flash);
                                    old = spi.output.pop_front().expect("tried to read missing flash follow output");
                                }
                            },
                            0xFE => {
                                // Disable chip
                                debug!(" [follow disable]");
                                spi.step(flash);
                            },
                            _ => {
                                panic!("Unknown follow address 0x{:02X}", a1);
                            }
                        }
                    } else {
                        old = flash[a];
                        if let Some(new) = new_opt {
                            flash[a] = new;
                        }
                    }
                },
                0x43 => debug!(" SCAR1L"),
                0x44 => debug!(" SCAR1M"),
                0x45 => {
                    debug!(" SCAR1H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(1);
                        }
                    }
                },
                0x48 => {
                    debug!(" SCAR2H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(2);
                        }
                    }
                },
                0x4B => {
                    debug!(" SCAR3H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(3);
                        }
                    }
                },
                0x4E => {
                    debug!(" SCAR4H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(4);
                        }
                    }
                },
                0x58 => debug!(" HINSTC1"),
                0x63 => debug!(" FLHCTRL3R"),
                _ => panic!("xram unimplemented SMFI register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // KBC
        0x1300 ... 0x13FF => {
            let base = 0x1300;
            let offset = address - base;
            debug!(" (KBC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" KBHICR"),
                0x02 => debug!(" KBIRQR"),
                _ => panic!("xram unimplemented KBC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // PMC
        0x1500 ... 0x15FF => {
            let base = 0x1500;
            let offset = address - base;
            debug!(" (PMC 0x{:02X}", offset);
            match offset {
                0x06 => debug!(" PM1CTL"),
                0x16 => debug!(" PM2CTL"),
                _ => panic!("xram unimplemented PMC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // GPIO
        0x1600 ... 0x16FF => {
            let base = 0x1600;
            let offset = address - base;
            debug!(" (GPIO 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" GCR"),

                0x01 => debug!(" GPDRA"),
                0x02 => debug!(" GPDRB"),
                0x03 => debug!(" GPDRC"),
                0x04 => debug!(" GPDRD"),
                0x05 => debug!(" GPDRE"),
                0x06 => debug!(" GPDRF"),
                0x07 => debug!(" GPDRG"),
                0x08 => debug!(" GPDRH"),
                0x09 => debug!(" GPDRI"),
                0x0A => debug!(" GPDRJ"),
                0x0D => debug!(" GPDRM"),

                0x10 ... 0x17 => debug!(" GPCRA{}", offset - 0x10),
                0x18 ... 0x1F => debug!(" GPCRB{}", offset - 0x18),
                0x20 ... 0x27 => debug!(" GPCRC{}", offset - 0x20),
                0x28 ... 0x2F => debug!(" GPCRD{}", offset - 0x28),
                0x30 ... 0x37 => debug!(" GPCRE{}", offset - 0x30),
                0x38 ... 0x3F => debug!(" GPCRF{}", offset - 0x38),
                0x40 ... 0x47 => debug!(" GPCRG{}", offset - 0x40),
                0x48 ... 0x4F => debug!(" GPCRH{}", offset - 0x48),
                0x50 ... 0x57 => debug!(" GPCRI{}", offset - 0x50),
                0x58 ... 0x5F => debug!(" GPCRJ{}", offset - 0x58),
                0xA0 ... 0xA6 => debug!(" GPCRM{}", offset - 0xA0),

                _ => panic!("xram unimplemented GPIO register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // PS/2
        0x1700 ... 0x17FF => {
            let base = 0x1700;
            let offset = address - base;
            debug!(" (PS/2 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" PSCTL1"),
                0x01 => debug!(" PSCTL2"),
                0x02 => debug!(" PSCTL3"),
                0x04 => debug!(" PSINT1"),
                0x05 => debug!(" PSINT2"),
                0x06 => debug!(" PSINT3"),
                _ => panic!("xram unimplemented PS/2 register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // ADC
        0x1900 ... 0x19FF => {
            let base = 0x1900;
            let offset = address - base;
            debug!(" (ADC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" ADCSTS"),
                0x01 => debug!(" ADCCFG"),
                0x04 => debug!(" VCH0CTL"),
                0x05 => debug!(" KDCTL"),
                0x06 => debug!(" VCH1CTL"),
                0x09 => debug!(" VCH2CTL"),
                0x0C => debug!(" VCH3CTL"),
                0x18 => debug!(" VCH0DATL"),
                0x19 => debug!(" VCH0DATM"),
                0x38 => debug!(" VCH4CTL"),
                _ => panic!("xram unimplemented ADC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // DAC
        0x1A00 ... 0x1AFF => {
            let base = 0x1A00;
            let offset = address - base;
            debug!(" (DAC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" UNKNOWN"),
                0x01 => debug!(" DACPDREG"),
                _ => panic!("xram unimplemented DAC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // SMBus
        0x1C00 ... 0x1CFF => {
            let base = 0x1C00;
            let offset = address - base;
            debug!(" (SMBUS 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" HOSTAA"),
                0x01 => debug!(" HOCTLA"),
                0x10 => debug!(" HOCTL2A"),
                0x11 => debug!(" HOSTAB"),
                0x12 => debug!(" HOCTLB"),
                0x21 => debug!(" HOCTL2B"),
                0x22 => debug!(" 4P7USL"),
                0x23 => debug!(" 4P0USL"),
                0x24 => debug!(" 300NS"),
                0x25 => debug!(" 250NS"),
                0x26 => debug!(" 25MS"),
                0x27 => debug!(" 45P3USL"),
                0x28 => debug!(" 45P3USH"),
                0x29 => debug!(" HOSTAC"),
                0x2A => debug!(" HOCTLC"),
                0x32 => debug!(" HOCTL2C"),
                _ => panic!("xram unimplemented SMBUS register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // KB Scan
        0x1D00 ... 0x1DFF => {
            let base = 0x1D00;
            let offset = address - base;
            debug!(" (KBSCAN 0x{:02X}", offset);
            match offset {
                0x02 => debug!(" KSOCTRL"),
                0x05 => debug!(" KSICTRLR"),
                _ => panic!("xram unimplemented KBSCAN register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // General Control
        0x2000 ... 0x20FF => {
            let base = 0x2000;
            let offset = address - base;
            debug!(" (GCTRL 0x{:02X}", offset);
            match offset {
                0x06 => debug!(" RSTS"),
                0x0A => debug!(" BADRSEL"),
                0x0B => debug!(" WNCKR"),
                0x0D => debug!(" SPCTRL1"),
                _ => panic!("xram unimplemented GCTRL register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // BRAM
        0x2200 ... 0x22FF => {
            let base = 0x2200;
            let offset = address - base;
            debug!(" (BRAM 0x{:02X})", offset);
        },
        _ => panic!("xram unimplemented register 0x{:04X}", address),
    }

    debug!(" load 0x{:02X}", old);
    if let Some(new) = new_opt {
        debug!(" store 0x{:02X}", new);
        mcu.store(Addr::XRam(address), new);
    }

    debug!("]");

    old
}
