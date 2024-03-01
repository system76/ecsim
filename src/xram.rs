// SPDX-License-Identifier: MIT

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

    // Bit masks for register access: Default is R/W
    let mut write_clear_mask = 0;
    let mut read_only_mask = 0;
    let mut write_only_mask = 0;

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
                0x05 => debug!(" FECBSR"),
                0x07 => debug!(" FMSSR"),
                0x20 => {
                    debug!(" SMECCS");
                    write_clear_mask = 0b0100_0000;
                    read_only_mask = 0b0001_1000;
                }
                0x32 => debug!(" FLHCTRL2R"),
                0x33 => debug!(" CACHDISR"),
                0x36 => debug!(" HCTRL2R"),
                0x3B => debug!(" ECINDAR0"),
                0x3C => debug!(" ECINDAR1"),
                0x3D => debug!(" ECINDAR2"),
                0x3E => {
                    debug!(" ECINDAR3");
                    read_only_mask = 0b0011_0000;
                    write_only_mask = 0b1100_0000;
                }
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

                    if ec.id == 0x5570 {
                        write_only_mask = 0b1000_0000;
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
                0x58 => {
                    debug!(" HINSTC1");
                    write_only_mask = 0b0100_1000;
                }
                0x5A => debug!(" HRAMWC"),
                0x5B => debug!(" HRAMW0BA"),
                0x5C => debug!(" HRAMW1BA"),
                0x5D => debug!(" HRAMW0AAS"),
                0x5E => debug!(" HRAMW1AAS"),
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

            // XXX: ISRx are R/WC if set to edge-triggered in IELMRx
            match offset {
                0x00 => {
                    debug!(" ISR0");
                    read_only_mask = 0b1111_1111;
                }
                0x01 => {
                    debug!(" ISR1");
                    read_only_mask = 0b1111_1111;
                }
                0x02 => {
                    debug!(" ISR2");
                    read_only_mask = 0b1111_1111;
                }
                0x03 => debug!(" ISR3"),
                0x05 => debug!(" IER1"),
                0x07 => debug!(" IER3"),
                0x10 => {
                    debug!(" IVECT");
                    read_only_mask = 0b1111_1111;
                }
                0x14 => {
                    debug!(" ISR4");
                    read_only_mask = 0b1111_1111;
                }
                0x15 => debug!(" IER4"),
                0x18 => {
                    debug!(" ISR5");
                    read_only_mask = 0b1111_1111;
                }
                0x19 => debug!(" IER5"),
                0x1C => {
                    debug!(" ISR6");
                    read_only_mask = 0b1111_1111;
                }
                0x1D => debug!(" IER6"),
                0x20 => {
                    debug!(" ISR7");
                    read_only_mask = 0b1111_1111;
                }
                0x21 => debug!(" IER7"),
                0x24 => {
                    debug!(" ISR8");
                    read_only_mask = 0b1111_1111;
                }
                0x25 => debug!(" IER8"),
                0x28 => {
                    debug!(" ISR9");
                    read_only_mask = 0b1111_1111;
                }
                0x29 => debug!(" IER9"),
                0x2C => {
                    debug!(" ISR10");
                    read_only_mask = 0b1111_1111;
                }
                0x2D => debug!(" IER10"),
                0x30 => {
                    debug!(" ISR11");
                    read_only_mask = 0b1111_1111;
                }
                0x31 => debug!(" IER11"),
                0x34 => {
                    debug!(" ISR12");
                    read_only_mask = 0b1111_1111;
                }
                0x35 => debug!(" IER12"),
                0x38 => {
                    debug!(" ISR13");
                    read_only_mask = 0b1111_1111;
                }
                0x39 => debug!(" IER13"),
                0x3C => {
                    debug!(" ISR14");
                    read_only_mask = 0b1111_1111;
                }
                0x3D => debug!(" IER14"),
                0x40 => {
                    debug!(" ISR15");
                    read_only_mask = 0b1111_1111;
                }
                0x41 => debug!(" IER15"),
                0x44 => {
                    debug!(" ISR16");
                    read_only_mask = 0b1111_1111;
                }
                0x45 => debug!(" IER16"),
                0x48 => {
                    debug!(" ISR17");
                    read_only_mask = 0b1111_1111;
                }
                0x49 => debug!(" IER17"),
                0x4C => {
                    debug!(" ISR18");
                    read_only_mask = 0b1111_1111;
                }
                0x4D => debug!(" IER18"),
                0x50 => {
                    debug!(" ISR19");
                    read_only_mask = 0b1111_1111;
                }
                0x51 => debug!(" IER19"),
                0x54 => {
                    debug!(" ISR20");
                    read_only_mask = 0b1111_1111;
                }
                0x55 => debug!(" IER20"),
                0x58 => {
                    debug!(" ISR21");
                    read_only_mask = 0b1111_1111;
                }
                0x59 => debug!(" IER21"),
                _ => panic!("xram unimplemented INTC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // E2CI
        0x1200 ..= 0x12FF => {
            let base = 0x1200;
            let offset = address - base;
            debug!(" (E2CI 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" IHIOA"),
                0x01 => debug!(" IHD"),
                0x02 => debug!(" LSIOHA"),
                0x04 => debug!(" IBMAE"),
                0x05 => {
                    debug!(" IBCTL");
                    read_only_mask = 0b0000_0100;
                }
                _ => panic!("xram unimplemented E2CI register 0x{:02X}", offset)
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
                0x02 => {
                    debug!(" KBIRQR");
                    read_only_mask = 0b1000_0000;
                }
                0x04 => {
                    debug!(" KBHISR");
                    read_only_mask = 0b0000_1011;
                }
                0x06 => {
                    debug!(" KBHIKDOR");
                    // Set output buffer full flag
                    mcu.xram[0x1304] |= 1 << 0;
                    write_only_mask = 0b1111_1111;
                },
                0x08 => {
                    debug!(" KBHIMDOR");
                    // Set output buffer full flag
                    mcu.xram[0x1304] |= 1 << 0;
                    write_only_mask = 0b1111_1111;
                },
                0x0A => {
                    debug!(" KBHIDIR");
                    // Clear input buffer full flag
                    mcu.xram[0x1304] &= !(1 << 1);
                    read_only_mask = 0b1111_1111;
                }
                _ => panic!("xram unimplemented KBC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // SWUC
        0x1400 ..= 0x14FF => {
            let base = 0x1400;
            let offset = address - base;
            debug!(" (SWUC 0x{:02X}", offset);
            match offset {
                0x08 => debug!(" SWCBALR"),
                0x0A => debug!(" SWCBAHR"),
                _ => panic!("xram unimplemented SWUC register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // PMC
        0x1500 ..= 0x15FF => {
            let base = 0x1500;
            let offset = address - base;
            debug!(" (PMC 0x{:02X}", offset);
            match offset {
                0x00 => {
                    debug!(" PM1STS");
                    read_only_mask = 0b0000_1011;
                }
                0x01 => {
                    debug!(" PM1DO");
                    // Set output buffer full flag
                    mcu.xram[0x1500] |= 1 << 0;
                    write_only_mask = 0b1111_1111;
                },
                0x04 => {
                    debug!(" PM1DI");
                    // Clear input buffer full flag
                    mcu.xram[0x1500] &= !(1 << 1);
                    read_only_mask = 0b1111_1111;
                }
                0x06 => debug!(" PM1CTL"),
                0x16 => debug!(" PM2CTL"),
                0x30 => {
                    debug!(" PM4STS");
                    read_only_mask = 0b0000_1011;
                }
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

                0x61 => {
                    debug!(" GPDRA");
                    read_only_mask = 0b1111_1111;
                }
                0x62 => {
                    debug!(" GPDRB");
                    read_only_mask = 0b1111_1111;
                }
                0x63 => {
                    debug!(" GPDRC");
                    read_only_mask = 0b1111_1111;
                }
                0x64 => {
                    debug!(" GPDRD");
                    read_only_mask = 0b1111_1111;
                }
                0x65 => {
                    debug!(" GPDRE");
                    read_only_mask = 0b1111_1111;
                }
                0x66 => {
                    debug!(" GPDRF");
                    read_only_mask = 0b1111_1111;
                }
                0x67 => {
                    debug!(" GPDRG");
                    read_only_mask = 0b1111_1111;
                }
                0x68 => {
                    debug!(" GPDRH");
                    read_only_mask = 0b1111_1111;
                }
                0x69 => {
                    debug!(" GPDRI");
                    read_only_mask = 0b1111_1111;
                }
                0x6A => {
                    debug!(" GPDRJ");
                    read_only_mask = 0b1111_1111;
                }
                0x6D => {
                    debug!(" GPDRM");
                    read_only_mask = 0b1111_1111;
                }

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
                0xA0 ..= 0xA7 => debug!(" GPCRM{}", offset - 0xA0),
                0xF8 => {
                    debug!(" GCR9");
                    if ec.id == 0x8587 {
                        write_only_mask = 0b0000_0100;
                    }
                }
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
                0x0A => {
                    debug!(" PSSTS3");
                    write_clear_mask = 0b0100_0000;
                    read_only_mask = 0b0011_1111;
                }
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
                0x40 => {
                    debug!(" CLK6MSEL");
                    read_only_mask = 0b0001_0000;
                }
                0x43 => debug!(" CTR3"),
                0x48 => {
                    debug!(" TSWCTLR");
                    write_clear_mask = 0b0000_1010;
                }
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
                0x00 => {
                    debug!(" ADCSTS");
                    write_clear_mask = 0b0000_0011;
                }
                0x01 => debug!(" ADCCFG"),
                0x04 => {
                    debug!(" VCH0CTL");
                    write_clear_mask = 0b1000_0000;
                }
                0x05 => debug!(" KDCTL"),
                0x06 => {
                    debug!(" VCH1CTL");
                    write_clear_mask = 0b1000_0000;
                }
                0x09 => {
                    debug!(" VCH2CTL");
                    write_clear_mask = 0b1000_0000;
                }
                0x0C => {
                    debug!(" VCH3CTL");
                    write_clear_mask = 0b1000_0000;
                }
                0x18 => {
                    debug!(" VCH0DATL");
                    read_only_mask = 0b1111_1111;
                }
                0x19 => {
                    debug!(" VCH0DATM");
                    read_only_mask = 0b0000_0011;
                }
                0x38 => {
                    debug!(" VCH4CTL");
                    write_clear_mask = 0b1000_0000;
                }
                0x3B => {
                    debug!(" VCH5CTL");
                    write_clear_mask = 0b1000_0000;
                }
                0x3E => {
                    debug!(" VCH6CTL");
                    write_clear_mask = 0b1000_0000;
                }
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
                0x00 => debug!(" DACCTRL"),
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
                0x00 => {
                    debug!(" HOSTAA");
                    write_clear_mask = 0b1111_1110;
                    read_only_mask = 0b0000_0001;
                }
                0x01 => {
                    debug!(" HOCTLA");
                    write_only_mask = 0b0110_0000;
                }
                0x02 => debug!(" HOCMDA"),
                0x03 => debug!(" TRASLAA"),
                0x04 => debug!(" D0REGA"),
                0x05 => debug!(" D1REGA"),
                0x06 => debug!(" HOBDBA"),
                0x07 => debug!(" PECERCA"),
                0x0A => {
                    debug!(" SMBPCTLA");
                    read_only_mask = 0b0000_0011;
                    write_only_mask = 0b0001_0000;
                }
                0x10 => {
                    debug!(" HOCTL2A");
                    if ec.id == 0x8587 {
                        write_only_mask = 0b1010_0000;
                    }
                }
                0x11 => {
                    debug!(" HOSTAB");
                    write_clear_mask = 0b1111_1110;
                    read_only_mask = 0b0000_0001;
                }
                0x12 => {
                    debug!(" HOCTLB");
                    write_only_mask = 0b0110_0000;
                }
                0x13 => debug!(" HOCMDB"),
                0x14 => debug!(" TRASLAB"),
                0x15 => debug!(" D0REGB"),
                0x16 => debug!(" D1REGB"),
                0x17 => debug!(" HOBDBB"),
                0x18 => debug!(" PECERCB"),
                0x1B => {
                    debug!(" SMBPCTLB");
                    read_only_mask = 0b0000_0011;
                    write_only_mask = 0b0001_0000;
                }
                0x21 => {
                    debug!(" HOCTL2B");
                    if ec.id == 0x8587 {
                        write_only_mask = 0b1010_0000;
                    }
                }
                0x22 => debug!(" 4P7USL"),
                0x23 => debug!(" 4P0USL"),
                0x24 => debug!(" 300NS"),
                0x25 => debug!(" 250NS"),
                0x26 => debug!(" 25MS"),
                0x27 => debug!(" 45P3USL"),
                0x28 => debug!(" 45P3USH"),
                0x29 => {
                    debug!(" HOSTAC");
                    write_clear_mask = 0b1111_1110;
                    read_only_mask = 0b0000_0001;
                }
                0x2A => {
                    debug!(" HOCTLC");
                    write_only_mask = 0b0110_0000;
                }
                0x2B => debug!(" HOCMDC"),
                0x2C => debug!(" TRASLAC"),
                0x2D => debug!(" D0REGC"),
                0x2E => debug!(" D1REGC"),
                0x2F => debug!(" HOBDBC"),
                0x30 => debug!(" PECERCC"),
                0x31 => {
                    debug!(" SMBPCTLC");
                    read_only_mask = 0b0000_0011;
                    write_only_mask = 0b0001_0000;
                }
                0x32 => {
                    debug!(" HOCTL2C");
                    if ec.id == 0x8587 {
                        write_only_mask = 0b1010_0000;
                    }
                }
                0x33 => debug!(" 4p7A4P0H"),
                0x35 => {
                    debug!(" HOSTAD");
                    write_clear_mask = 0b1111_1110;
                    read_only_mask = 0b0000_0001;
                }
                0x36 => {
                    debug!(" HOCTLD");
                    write_only_mask = 0b0110_0000;
                }
                0x37 => debug!(" HOCMDD"),
                0x38 => debug!(" TRASLAD"),
                0x39 => debug!(" D0REGD"),
                0x3A => debug!(" D1REGD"),
                0x3B => debug!(" HOBDBD"),
                0x3C => debug!(" PECERCD"),
                0x3D => {
                    debug!(" SMBPCTLD");
                    read_only_mask = 0b0000_0011;
                    write_only_mask = 0b0001_0000;
                }
                0x3E => {
                    debug!(" HOCTL2D");
                    if ec.id == 0x8587 {
                        write_only_mask = 0b1010_0000;
                    }
                }
                0x41 => debug!(" SCLKTSB"),
                0xA0 if ec.id == 0x5570 => {
                    debug!(" HOSTAE");
                    write_clear_mask = 0b1111_1110;
                    read_only_mask = 0b0000_0001;
                }
                0xA1 if ec.id == 0x5570 => {
                    debug!(" HOCTLE");
                    write_only_mask = 0b0110_0000;
                }
                0xA2 if ec.id == 0x5570 => debug!(" HOCMDE"),
                0xA3 if ec.id == 0x5570 => debug!(" TRASLAE"),
                0xA4 if ec.id == 0x5570 => debug!(" D0REGE"),
                0xA6 if ec.id == 0x5570 => debug!(" D1REGE"),
                0xA7 if ec.id == 0x5570 => debug!(" HOBDBE"),
                0xA8 if ec.id == 0x5570 => debug!(" PECERCE"),
                0xA9 if ec.id == 0x5570 => {
                    debug!(" SMBPCTLE");
                    read_only_mask = 0b0000_0011;
                    write_only_mask = 0b0001_0000;
                }
                0xAA if ec.id == 0x5570 => debug!(" HOCTL2E"),
                0xAB if ec.id == 0x5570 => debug!(" SCLKTS_E"),
                0xB0 if ec.id == 0x5570 => {
                    debug!(" HOSTAF");
                    write_clear_mask = 0b1111_1110;
                    read_only_mask = 0b0000_0001;
                }
                0xB1 if ec.id == 0x5570 => {
                    debug!(" HOCTLF");
                    write_only_mask = 0b0110_0000;
                }
                0xB2 if ec.id == 0x5570 => debug!(" HOCMDF"),
                0xB3 if ec.id == 0x5570 => debug!(" TRASLAF"),
                0xB4 if ec.id == 0x5570 => debug!(" D0REGF"),
                0xB6 if ec.id == 0x5570 => debug!(" D1REGF"),
                0xB7 if ec.id == 0x5570 => debug!(" HOBDBF"),
                0xB8 if ec.id == 0x5570 => debug!(" PECERCF"),
                0xB9 if ec.id == 0x5570 => {
                    debug!(" SMBPCTLF");
                    read_only_mask = 0b0000_0011;
                    write_only_mask = 0b0001_0000;
                }
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
                0x04 => {
                    debug!(" KSI");
                    read_only_mask = 0b1111_1111;
                }
                0x05 => debug!(" KSICTRLR"),
                0x06 => debug!(" KSIGCTRL"),
                0x07 => debug!(" KSIGOEN"),
                0x08 => debug!(" KSIGDAT"),
                0x09 => {
                    debug!(" KSIGDMRR");
                    read_only_mask = 0b1111_1111;
                }
                0x0A => debug!(" KSOHGCTRL"),
                0x0B => debug!(" KSOHGOEN"),
                0x0C => {
                    debug!(" KSOHGDMRR");
                    read_only_mask = 0b1111_1111;
                }
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
                0x03 => debug!(" PLLCTRL"),
                0x05 => {
                    debug!(" CGCTRL3");
                    write_only_mask = 0b0100_0000;
                }
                0x06 => debug!(" PLLFREQR"),
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
                0x00 => {
                    debug!(" ECHIPID1");
                    read_only_mask = 0b1111_1111;
                }
                0x01 => {
                    debug!(" ECHIPID2");
                    read_only_mask = 0b1111_1111;
                }
                0x02 => {
                    debug!(" ECHIPVER");
                    read_only_mask = 0b1111_1111;
                }
                0x06 => {
                    debug!(" RSTS");
                    if ec.id == 0x8587 {
                        read_only_mask = 0b0000_0011;
                    }
                    if ec.id == 0x5570 {
                        write_clear_mask = 0b0000_0011;
                    }
                }
                0x0A => debug!(" BADRSEL"),
                0x0B => {
                    debug!(" WNCKR");
                    write_only_mask = 0b1111_1111;
                }
                0x0D => debug!(" SPCTRL1"),
                0x30 => {
                    debug!(" P80H81HS");
                    write_clear_mask = 0b0000_0001;
                }
                0x31 => debug!(" P80HDR"),
                0x32 => debug!(" P81HDR"),
                _ => panic!("xram unimplemented GCTRL register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // BRAM
        0x2200 ..= 0x22FF => {
            let base = 0x2200;
            let _offset = address - base;
            debug!(" (BRAM 0x{:02X})", _offset);
        },
        // PECI
        0x3000 ..= 0x30FF => {
            let base = 0x3000;
            let offset = address - base;
            debug!(" (PECI 0x{:02X}", offset);
            match offset {
                0x00 => {
                    debug!(" HOSTAR");
                    write_clear_mask = 0b1110_1110;
                    read_only_mask = 0b0000_0001;
                }
                0x01 => {
                    debug!(" HOCTLR");
                    write_only_mask = 0b0010_0001;
                }
                0x02 => debug!(" HOCMDR"),
                0x03 => debug!(" HOTRADDR"),
                0x04 => debug!(" HOWRLR"),
                0x05 => debug!(" HORDLR"),
                0x06 => debug!(" HOWRDR"),
                0x07 => debug!(" HORDDR"),
                0x08 => debug!(" HOCTL2R"),
                0x09 => {
                    debug!(" RWFCSV");
                    read_only_mask = 0b1111_1111;
                }
                0x0E => debug!(" PADCTLR"),
                _ => panic!("xram unimplemented PECI register 0x{:02X}", offset)
            }
            debug!(")");
        },
        // eSPI
        0x3100 ..= 0x32FF if ec.id == 0x5570 => {
            let base = 0x3100;
            let offset = address - base;
            debug!(" (eSPI 0x{:02X}", offset);
            match offset {
                // Peripheral
                0x04 => {
                    debug!(" General Capabilities and Configurations 3");
                    read_only_mask = 0b1101_1111;
                }
                0x05 => {
                    debug!(" General Capabilities and Configurations 2");
                    read_only_mask = 0b0111_0000;
                }
                0x06 => {
                    debug!(" General Capabilities and Configurations 1");
                    read_only_mask = 0b1111_0000;
                }
                0x07 => {
                    debug!(" General Capabilities and Configurations 0");
                    read_only_mask = 0b1111_1111;
                }
                0x14 => debug!(" Channel 3 Capabilities and Configurations 3"),
                0x15 => debug!(" Channel 3 Capabilities and Configurations 2"),
                0x16 => {
                    debug!(" Channel 3 Capabilities and Configurations 1");
                    read_only_mask = 0b0111_1111;
                }
                0x17 => {
                    debug!(" Channel 3 Capabilities and Configurations 0");
                    read_only_mask = 0b1111_1111;
                }
                0xA1 => debug!(" ESGCTRL1"),
                0xA2 => debug!(" ESGCTRL2"),
                0xA3 => debug!(" ESGCTRL3"),
                0xB0 => debug!(" ESUCTRL0"),
                0xB1 => debug!(" ESUCTRL1"),
                0xB2 => debug!(" ESUCTRL2"),
                0xB3 => debug!(" ESUCTRL3"),
                0xB6 => debug!(" ESUCTRL6"),
                0xB7 => debug!(" ESUCTRL7"),
                0xB8 => debug!(" ESUCTRL8"),
                0xC0 => debug!(" ESOCTRL0"),
                0xC1 => debug!(" ESOCTRL1"),
                0xC4 => debug!(" ESOCTRL4"),
                // Virtual wire
                0x190 => debug!(" VWCTRL0"),
                _ => panic!("xram unimplemented eSPI register 0x{:02X}", offset)
            }
            debug!(")");
        },
        0x8000 ..= 0x97FF if ec.id == 0x5570 => {
            let base = 0x8000;
            let _offset = address - base;
            debug!(" (SRAM 0x{:02X})", _offset);
        }
        _ => panic!("xram unimplemented register 0x{:04X}", address),
    }

    old &= !write_only_mask;
    debug!(" load 0x{:02X}", old);
    if let Some(new) = new_opt {
        debug!(" store 0x{:02X}", new);

        let rwc = ((old & !new) & write_clear_mask) | (new & !write_clear_mask);
        let ro = old | (new & !read_only_mask);
        let value = rwc & ro;

        mcu.store(Addr::XRam(address), value);
    }

    debug!("]");

    old
}
