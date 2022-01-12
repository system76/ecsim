// SPDX-License-Identifier: MIT

use area8051::{Addr, Mem};

use super::Ec;

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
        // GPIO
        0x0000 ..= 0x00FF | 0xFC00 ..= 0xFC7F => {
            let offset = if address >= 0xFC00 {
                address - 0xFC00
            } else {
                address
            };
            debug!(" (GPIO 0x{:02X}", offset);
            match offset {
                0x00 ..= 0x0F => debug!(" GPIOFS{:02X}", offset * 8),
                0x10 ..= 0x1F => debug!(" GPIOOE{:02X}", (offset - 0x10) * 8),
                0x20 ..= 0x2F => debug!(" GPIOD{:02X}", (offset - 0x20) * 8),
                0x30 ..= 0x3F => debug!(" GPIOIN{:02X}", (offset - 0x30) * 8),
                0x40 ..= 0x4F => debug!(" GPIOPU{:02X}", (offset - 0x40) * 8),
                0x50 ..= 0x5F => debug!(" GPIOOD{:02X}", (offset - 0x50) * 8),
                0x60 ..= 0x6F => debug!(" GPIOIE{:02X}", (offset - 0x60) * 8),
                0x70 => debug!(" GPIOMISC"),
                0x71 => debug!(" GPIOMISC2"),
                0x76 => debug!(" AOODC"),
                0x7B => debug!(" GPIOMISC3"),
                0x80 ..= 0x8F => debug!(" GPIOLV{:02X}", (offset - 0x80) * 8),
                0x90 ..= 0x9F => debug!(" GPIODC{:02X}", (offset - 0x90) * 8),
                _ => panic!("xram unimplemented GPIO register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Keyboard controller
        0x0100 ..= 0x011F => {
            let offset = address - 0x0100;
            debug!(" (KBC 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" KBCCB"),
                0x01 => debug!(" KBCCFG"),
                0x02 => debug!(" KBCPF"),
                0x03 => debug!(" KBCHWEN"),
                0x06 => debug!(" KBCSTS"),
                _ => panic!("xram unimplemented KBC register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Internal keyboard matrix
        0x0300 ..= 0x030F => {
            let offset = address - 0x0300;
            debug!(" (IKB 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" IKBCFG"),
                0x01 => debug!(" IKBLED"),
                0x03 => debug!(" IKBIE"),
                0x04 => debug!(" IKBPF"),
                0x0A => debug!(" IKBSDB"),
                0x0D => debug!(" IKBSADB"),
                _ => panic!("xram unimplemented IKB register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Fan controller 0
        0x0A00 ..= 0x0A1F => {
            let offset = address - 0x0A00;
            debug!(" (FAN0 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" FANCFG0"),
                0x01 => debug!(" FANSTS0"),
                0x10 => debug!(" FANDFT0"),
                _ => panic!("xram unimplemented FAN0 register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // General purpose timer
        0x0C00 ..= 0x0C0F => {
            let offset = address - 0x0C00;
            debug!(" (GPT 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" GPTCFG"),
                0x03 => debug!(" GPT0"),
                0x05 => debug!(" GPT1"),
                0x06 => debug!(" GPT2H"),
                0x07 => debug!(" GPT2L"),
                0x08 => debug!(" GPT3H"),
                0x09 => debug!(" GPT3L"),
                _ => panic!("xram unimplemented GPT register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Watchdog timer
        0x0E00 ..= 0x0EFF => {
            let offset = address - 0x0E00;
            debug!(" (WDT 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" WDTCFG"),
                0x01 => debug!(" WDTPF"),
                0x02 => debug!(" WDTM0902"),
                0x0A => debug!(" CLK32CR"),
                0x0E => debug!(" WDTFCR0"),
                0x0F => debug!(" WDTFCR1"),
                0x11 => debug!(" WDTCMR"),
                _ => panic!("xram unimplemented WDT register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Low pin count
        0x0F00 ..= 0x0F7F => {
            let offset = address - 0x0F00;
            debug!(" (LPC 0x{:02X}", offset);
            match offset {
                0x10 => debug!(" LPCSCFG"),
                0x11 => debug!(" LPCSIRQ"),
                0x12 => debug!(" LPCIBAH"),
                0x13 => debug!(" LPCIBAL"),
                0x15 => debug!(" LPCCFG"),
                0x16 => debug!(" LPCX0BAH"),
                0x17 => debug!(" LPCX0BAL"),
                0x18 => debug!(" LPCEBAH"),
                0x19 => debug!(" LPCEBAL"),
                0x1D => debug!(" LPC68CFG"),
                0x1E => debug!(" LPC68CSR"),
                0x21 => debug!(" LPCFPCFG"),
                0x2D => debug!(" LPCXBA15"),
                0x2E => debug!(" LPCXBA31"),
                0x2F => debug!(" LPCXBA23"),
                0x30 => debug!(" LPCBRP0"),
                0x31 => debug!(" LPCBRP1"),
                0x32 => debug!(" LPCBWP0"),
                0x33 => debug!(" LPCBWP1"),
                0x34 => debug!(" LPCSRP0"),
                0x35 => debug!(" LPCSRP1"),
                0x36 => debug!(" LPCSWP0"),
                0x37 => debug!(" LPCSWP1"),
                0x42 => debug!(" LCMB1SCR"),
                0x43 => debug!(" LPCMB1BAH"),
                0x44 => debug!(" LPCMB1BAL"),
                0x45 => debug!(" LPCMB1EBA"),
                0x46 => debug!(" LPCMB1WPC1"),
                0x47 => debug!(" LPCMB1WPC0"),
                0x48 => debug!(" LPCMB1RPC1"),
                0x49 => debug!(" LPCMB1RPC0"),
                0x4E => debug!(" LPC78STA"),
                0x50 => debug!(" LPC68H"),
                0x51 => debug!(" LPC68L"),
                _ => panic!("xram unimplemented LPC register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // X-bus interface
        0x1000 ..= 0x107F => {
            let offset = address - 0x1000;
            debug!(" (XBI 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" XBISEG0"),
                0x01 => debug!(" XBISEG0E"),
                0x03 => debug!(" XBISEG1E"),
                0x05 => debug!(" XBISEG2E"),
                0x08 => debug!(" XBISEG0DV"),
                0x09 => debug!(" XBISEG0EDV"),
                0x1F => debug!(" SPICFG"),
                //TODO: BOOT TIMER?
                0x21 => debug!(" SPIMSCR"),
                //TODO: run code from xram
                0x26 => debug!(" XBICFG2"),
                0x28 => debug!(" EFA0"),
                0x29 => debug!(" EFA1"),
                0x2A => debug!(" EFA2"),
                //TODO: read/write to embedded flash
                0x2B => debug!(" EFDAT"),
                0x2C => debug!(" EFCMD"),
                0x2D => debug!(" EFCFG"),
                0x5E => debug!(" SPISIZE"),
                _ => panic!("xram unimplemented XBI register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // PS/2 interface
        0x1300 ..= 0x131F => {
            let offset = address - 0x1300;
            debug!(" (PS2 0x{:02X}", offset);
            match offset {
                0x00 => debug!(" PS2CCTL"),
                0x03 => debug!(" PS2INTEN"),
                0x04 => debug!(" PS2INTPF"),
                0x0A => debug!(" PS2SWRST"),
                0x0C => debug!(" PS2CGC"),
                _ => panic!("xram unimplemented PS2 register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Embedded controller
        0x1400 ..= 0x14FF | 0xFF00 ..= 0xFF2F => {
            let offset = if address >= 0xFF00 {
                address - 0xFF00
            } else {
                address - 0x1400
            };
            debug!(" (EC 0x{:02X}", offset);
            match offset {
                0x01 => debug!(" ECFV"),
                0x02 => debug!(" ECHA"),
                0x03 => debug!(" SCICFG"),
                0x04 => debug!(" ECCFG"),
                0x05 => debug!(" SCIE0"),
                0x0C => debug!(" PMUCFG"),
                0x0D => debug!(" CLKCFG"),
                0x14 => debug!(" PXCFG"),
                0x15 => debug!(" ADDAEN"),
                0x18 => debug!(" ADCTRL"),
                0x1D => debug!(" ECSTS"),
                0x20 => debug!(" ECMISC"),
                0x22 => debug!(" EDIF"),
                0x26 => debug!(" ADCIE"),
                0x43 => debug!(" ADEN2"),
                _ => panic!("xram unimplemented EC register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        }
        // General purpose wakeup
        0x1500 ..= 0x157F => {
            let offset = address - 0x1500;
            debug!(" (GPWU 0x{:02X}", offset);
            match offset {
                0x30 ..= 0x3F => debug!(" GPWUEN{:02X}", (offset - 0x30) * 8),
                0x40 ..= 0x4F => debug!(" GPWUPF{:02X}", (offset - 0x40) * 8),
                0x50 ..= 0x5F => debug!(" GPWUPS{:02X}", (offset - 0x50) * 8),
                0x60 ..= 0x6F => debug!(" GPWUEL{:02X}", (offset - 0x60) * 8),
                0x70 ..= 0x7F => debug!(" GPWUCHG{:02X}", (offset - 0x70) * 8),
                _ => panic!("xram unimplemented GPWU register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // FSM master 1
        0x1700 ..= 0x173F => {
            let offset = address - 0x1700;
            debug!(" (FSM1 0x{:02X}", offset);
            match offset {
                0x10 => debug!(" FSMB1CFG"),
                0x13 => debug!(" FSMB1OFH"),
                0x14 => debug!(" FSMB1OFL"),
                0x15 => debug!(" FSMB1IE"),
                0x1A => debug!(" FSMB1ADR"),
                0x1C ..= 0x3B => debug!(" FSMB1DAT[{}]", offset - 0x1C),
                _ => panic!("xram unimplemented FSM1 register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // I2C Device Interface
        0x1B00 ..= 0x1B2F => {
            let offset = address - 0x1B00;
            debug!(" (I2CD 0x{:02X}", offset);
            match offset {
                0x10 => debug!(" I2CDCFG"),
                0x14 => debug!(" I2CDADR"),
                _ => panic!("xram unimplemented I2CD register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // FSM master 2
        0x2100 ..= 0x213F => {
            let offset = address - 0x2100;
            debug!(" (FSM2 0x{:02X}", offset);
            match offset {
                0x10 => debug!(" FSMB2CFG"),
                0x13 => debug!(" FSMB2OFH"),
                0x14 => debug!(" FSMB2OFL"),
                0x15 => debug!(" FSMB2IE"),
                _ => panic!("xram unimplemented FSM2 register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // FSM master 3
        0x2200 ..= 0x223F => {
            let offset = address - 0x2200;
            debug!(" (FSM3 0x{:02X}", offset);
            match offset {
                0x10 => debug!(" FSMB3CFG"),
                0x13 => debug!(" FSMB3OFH"),
                0x14 => debug!(" FSMB3OFL"),
                0x15 => debug!(" FSMB3IE"),
                _ => panic!("xram unimplemented FSM3 register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Direct Memory Access
        0x2500 ..= 0x252F => {
            let offset = address - 0x2500;
            debug!(" (DMA 0x{:02X}", offset);
            match offset {
                //TODO: DMA can access all kinds of other xram
                0x00 => debug!(" DMACFG"),
                0x10 => debug!(" DMAC0CFG"),
                0x20 => debug!(" DMAC1CFG"),
                _ => panic!("xram unimplemented DMA register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Embedded Flash Protect
        0x2700 ..= 0x278F => {
            let offset = address - 0x2700;
            debug!(" (EFP 0x{:02X}", offset);
            match offset {
                0x38 => debug!(" EFPR7SAH"),
                0x39 => debug!(" EFPR7SAL"),
                0x3A => debug!(" EFPR7EAH"),
                0x3B => debug!(" EFPR7EAL"),
                0x3C => debug!(" EFPR7CTL"),
                _ => panic!("xram unimplemented EFP register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Power-latch and voltage comparator
        0x2800 ..= 0x2803 => {
            let offset = address - 0x2800;
            debug!(" (PAREG 0x{:02X}", offset);
            match offset {
                0x02 => debug!(" PA2REG"),
                _ => panic!("xram unimplemented PAREG register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Scratch SRAM
        0xE800 ..= 0xFBFF => {
            let offset = address - 0xE800;
            debug!(" (SRAM 0x{:02X})", offset);
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
