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
        0x0000 ..= 0x0FFF => {
            debug!(" (SRAM)");
        },
        0x8000 ..= 0x97FF if ec.id == 0x5570 => {
            debug!(" (SRAM)");
            //TODO: SRAM is double mapped from 0x8000 - 0x8FFF
        },
        // SMFI
        0x1000 ..= 0x10FF => {
            let base = 0x1000;
            let offset = address - base;
            debug!(" (SMFI 0x{:02X}", offset);

            let mut scar_dma = |scar| {
                let (reg, base, size) = ec.scar()[scar];

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
                0x32 => debug!(" FLHCTRL2R"),
                0x33 => debug!(" CACHDISR"),
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
                    let (flash, flash_name): (&mut [u8], &str) = match (a3 >> 6) & 0b11 {
                        0b00 | 0b11 => {
                            (&mut xmem, "external")
                        },
                        0b01 => {
                            (&mut mcu.pmem, "internal")
                        },
                        unknown => {
                            panic!("unknown ECIND flash chip 0b{:02b}", unknown);
                        }
                    };
                    debug!(" ({})]", flash_name);

                    if a3 & 0xF == 0xF {
                        match a1 {
                            0xFD => {
                                // Enable chip, send or receive
                                debug!(" [follow enable]");
                                if let Some(new) = new_opt {
                                    spi.input.push_back(new);
                                } else {
                                    spi.step(flash, flash_name);
                                    old = spi.output.pop_front().expect("tried to read missing flash follow output");
                                }
                            },
                            0xFE => {
                                // Disable chip
                                debug!(" [follow disable]");
                                spi.step(flash, flash_name);
                            },
                            _ => {
                                panic!("Unknown follow address 0x{:02X}", a1);
                            }
                        }
                    } else {
                        let i = a & 0xFFFFFF;
                        old = flash[i];
                        if let Some(new) = new_opt {
                            flash[i] = new;
                        }
                    }
                },
                0x40 => debug!(" SCAR0L"),
                0x41 => debug!(" SCAR0M"),
                0x42 => {
                    debug!(" SCAR0H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(0);
                        }
                    }
                },
                0x43 if ec.id == 0x8587 => debug!(" SCAR1L"),
                0x44 if ec.id == 0x8587 => debug!(" SCAR1M"),
                0x45 if ec.id == 0x8587 => {
                    debug!(" SCAR1H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(1);
                        }
                    }
                },
                0x46 if ec.id == 0x8587 => debug!(" SCAR2L"),
                0x47 if ec.id == 0x8587 => debug!(" SCAR2M"),
                0x48 if ec.id == 0x8587 => {
                    debug!(" SCAR2H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(2);
                        }
                    }
                },
                0x49 if ec.id == 0x8587 => debug!(" SCAR3L"),
                0x4A if ec.id == 0x8587 => debug!(" SCAR3M"),
                0x4B if ec.id == 0x8587 => {
                    debug!(" SCAR3H");
                    if let Some(new) = new_opt {
                        if old & 0x80 != 0 && new & 0x80 == 0 {
                            scar_dma(3);
                        }
                    }
                },
                0x4C if ec.id == 0x8587 => debug!(" SCAR4L"),
                0x4D if ec.id == 0x8587 => debug!(" SCAR4M"),
                0x4E if ec.id == 0x8587 => {
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
        // INTC
        0x1100 ..= 0x11FF => {
            let base = 0x1100;
            let offset = address - base;
            debug!(" (INTC 0x{:02X}", offset);
            match offset {
                0x01 => debug!(" ISR1"),
                0x05 => debug!(" IER1"),
                0x07 => debug!(" IER3"),
                0x10 => debug!(" IVECT"),
                _ => panic!("xram unimplemented INTC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // KBC
        0x1300 ..= 0x13FF => {
            let base = 0x1300;
            let offset = address - base;
            debug!(" (KBC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" KBHICR"),
                0x02 => debug!(" KBIRQR"),
                0x04 => debug!(" KBHISR"),
                0x06 => {
                    debug!(" KBHIKDOR");
                    //TODO: Enforce write-only
                    // Set output buffer full flag
                    mcu.xram[0x1304] |= 1 << 0;
                },
                0x08 => {
                    debug!(" KBHIMDOR");
                    //TODO: Enforce write-only
                    // Set output buffer full flag
                    mcu.xram[0x1304] |= 1 << 0;
                },
                0x0A => {
                    debug!(" KBHIDIR");
                    //TODO: Enforce read-only
                    // Clear input buffer full flag
                    mcu.xram[0x1304] &= !(1 << 1);
                }
                _ => panic!("xram unimplemented KBC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // PMC
        0x1500 ..= 0x15FF => {
            let base = 0x1500;
            let offset = address - base;
            debug!(" (PMC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" PM1STS"),
                0x01 => {
                    debug!(" PM1DO");
                    //TODO: Enforce write-only
                    // Set output buffer full flag
                    mcu.xram[0x1500] |= 1 << 0;
                },
                0x04 => {
                    debug!(" PM1DI");
                    //TODO: Enforce read-only
                    // Clear input buffer full flag
                    mcu.xram[0x1500] &= !(1 << 1);
                }
                0x06 => debug!(" PM1CTL"),
                0x16 => debug!(" PM2CTL"),
                0x30 => debug!(" PM4STS"),
                _ => panic!("xram unimplemented PMC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // GPIO
        0x1600 ..= 0x16FF => {
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

                0x61 => debug!(" GPDRA"),
                0x62 => debug!(" GPDRB"),
                0x63 => debug!(" GPDRC"),
                0x64 => debug!(" GPDRD"),
                0x65 => debug!(" GPDRE"),
                0x66 => debug!(" GPDRF"),
                0x67 => debug!(" GPDRG"),
                0x68 => debug!(" GPDRH"),
                0x69 => debug!(" GPDRI"),
                0x6A => debug!(" GPDRJ"),
                0x6D => debug!(" GPDRM"),

                0x71 => debug!(" GPOTA"),
                0x72 => debug!(" GPOTB"),
                0x73 => debug!(" GPOTC"),
                0x74 => debug!(" GPOTD"),
                0x75 => debug!(" GPOTE"),
                0x76 => debug!(" GPOTF"),
                0x77 => debug!(" GPOTG"),
                0x78 => debug!(" GPOTH"),
                0x79 => debug!(" GPOTI"),
                0x7A => debug!(" GPOTJ"),
                0x7D => debug!(" GPOTM"),

                0x10 ..= 0x17 => debug!(" GPCRA{}", offset - 0x10),
                0x18 ..= 0x1F => debug!(" GPCRB{}", offset - 0x18),
                0x20 ..= 0x27 => debug!(" GPCRC{}", offset - 0x20),
                0x28 ..= 0x2F => debug!(" GPCRD{}", offset - 0x28),
                0x30 ..= 0x37 => debug!(" GPCRE{}", offset - 0x30),
                0x38 ..= 0x3F => debug!(" GPCRF{}", offset - 0x38),
                0x40 ..= 0x47 => debug!(" GPCRG{}", offset - 0x40),
                0x48 ..= 0x4F => debug!(" GPCRH{}", offset - 0x48),
                0x50 ..= 0x57 => debug!(" GPCRI{}", offset - 0x50),
                0x58 ..= 0x5F => debug!(" GPCRJ{}", offset - 0x58),
                0xA0 ..= 0xA6 => debug!(" GPCRM{}", offset - 0xA0),

                0xF0 ..= 0xFE => debug!(" GCR{}", offset - 0xF0 + 1),
                0xE0 ..= 0xE2 => debug!(" GCR{}", offset - 0xE0 + 16),
                0xE4 ..= 0xE8 if ec.id == 0x5570 => debug!(" GCR{}", offset - 0xE4 + 19),

                _ => panic!("xram unimplemented GPIO register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // PS/2
        0x1700 ..= 0x17FF => {
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
                0x0A => debug!(" PSSTS3"),
                _ => panic!("xram unimplemented PS/2 register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // PWM
        0x1800 ..= 0x18FF => {
            let base = 0x1800;
            let offset = address - base;
            debug!(" (PWM 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" C0CPRS"),
                0x01 => debug!(" CTR0"),
                0x02 ..= 0x09 => debug!(" DCR{}", offset - 0x02),
                0x0B => debug!(" PCFSR"),
                0x0C => debug!(" PCSSGL"),
                0x0D => debug!(" PCSSGH"),
                0x0F => debug!(" PCSGR"),
                0x23 => debug!(" ZTIER"),
                0x27 => debug!(" C4CPRS"),
                0x2B => debug!(" C6CPRS"),
                0x2C => debug!(" C6MCPRS"),
                0x2D => debug!(" C7CPRS"),
                0x2E => debug!(" C7MCPRS"),
                0x40 => debug!(" CLK6MSEL"),
                0x43 => debug!(" CTR3"),
                0x48 => debug!(" TSWCTLR"),
                _ => panic!("xram unimplemented PWM register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // ADC
        0x1900 ..= 0x19FF => {
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
                0x3B => debug!(" VCH5CTL"),
                0x3E => debug!(" VCH6CTL"),
                _ => panic!("xram unimplemented ADC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // DAC
        0x1A00 ..= 0x1AFF => {
            let base = 0x1A00;
            let offset = address - base;
            debug!(" (DAC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" UNKNOWN"),
                0x01 => debug!(" DACPDREG"),
                0x04 => debug!(" DACDAT2"),
                _ => panic!("xram unimplemented DAC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // SMBus
        0x1C00 ..= 0x1CFF => {
            let base = 0x1C00;
            let offset = address - base;
            debug!(" (SMBUS 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" HOSTAA"),
                0x01 => debug!(" HOCTLA"),
                0x02 => debug!(" HOCMDA"),
                0x03 => debug!(" TRASLAA"),
                0x04 => debug!(" D0REGA"),
                0x05 => debug!(" D1REGA"),
                0x06 => debug!(" HOBDBA"),
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
                0x35 => debug!(" HOSTAD"),
                0x36 => debug!(" HOCTLD"),
                0x3E => debug!(" HOCTL2D"),
                0x41 => debug!(" SCLKTSB"),
                0xA0 if ec.id == 0x5570 => debug!(" HOSTAE"),
                0xA1 if ec.id == 0x5570 => debug!(" HOCTLE"),
                0xA3 if ec.id == 0x5570 => debug!(" TRASLAE"),
                0xA7 if ec.id == 0x5570 => debug!(" HOBDBE"),
                0xAA if ec.id == 0x5570 => debug!(" HOCTL2E"),
                0xB0 if ec.id == 0x5570 => debug!(" HOSTAF"),
                0xB1 if ec.id == 0x5570 => debug!(" HOCTLF"),
                0xBA if ec.id == 0x5570 => debug!(" HOCTL2F"),
                _ => panic!("xram unimplemented SMBUS register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // KB Scan
        0x1D00 ..= 0x1DFF => {
            let base = 0x1D00;
            let offset = address - base;
            debug!(" (KBSCAN 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" KSOL"),
                0x01 => {
                    debug!(" KSOH1");
                    if let Some(new) = new_opt {
                        if new & 1 == 0 {
                            let byte = mcu.xram[0x1D00];
                            print!("{}", byte as char);
                        }
                    }
                },
                0x02 => debug!(" KSOCTRL"),
                0x03 => debug!(" KSOH2"),
                0x04 => debug!(" KSI"),
                0x05 => debug!(" KSICTRLR"),
                0x06 => debug!(" KSIGCTRL"),
                0x07 => debug!(" KSIGOEN"),
                0x08 => debug!(" KSIGDAT"),
                0x09 => debug!(" KSIGDMRR"),
                0x0A => debug!(" KSOHGCTRL"),
                0x0B => debug!(" KSOHGOEN"),
                0x0C => debug!(" KSOHGDMRR"),
                0x0D => debug!(" KSOLGCTRL"),
                0x0E => debug!(" KSOLGOEN"),
                0x0F => debug!(" KSOLGDMRR"),
                _ => panic!("xram unimplemented KBSCAN register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // EC power management
        0x1E00 ..= 0x1EFF => {
            let base = 0x1E00;
            let offset = address - base;
            debug!(" (ECPM 0x{:02X}", offset);
            match offset {
                0x02 => debug!(" CGCTRL2"),
                0x05 => debug!(" CGCTRL3"),
                0x09 => debug!(" CGCTRL4"),
                _ => panic!("xram unimplemented ECPM register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // General Control
        0x2000 ..= 0x20FF => {
            let base = 0x2000;
            let offset = address - base;
            debug!(" (GCTRL 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" ECHIPID1"),
                0x01 => debug!(" ECHIPID2"),
                0x02 => debug!(" ECHIPVER"),
                0x06 => debug!(" RSTS"),
                0x0A => debug!(" BADRSEL"),
                0x0B => debug!(" WNCKR"),
                0x0D => debug!(" SPCTRL1"),
                _ => panic!("xram unimplemented GCTRL register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // BRAM
        0x2200 ..= 0x22FF => {
            let base = 0x2200;
            let offset = address - base;
            debug!(" (BRAM 0x{:02X})", offset);
        },
        // PECI
        0x3000 ..= 0x30FF => {
            let base = 0x3000;
            let offset = address - base;
            debug!(" (PECI 0x{:02X}", offset);
            match offset {
                0x08 => debug!(" HOCTL2R"),
                0x0E => debug!(" PADCTLR"),
                _ => panic!("xram unimplemented PECI register 0x{:02X}", offset)
            }
            debug!(")");
        },
        0x8000 ..= 0x97FF if ec.id == 0x5570 => {
            let base = 0x8000;
            let offset = address - base;
            debug!(" (SRAM 0x{:02X})", offset);
        }
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
