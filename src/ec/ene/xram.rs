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
                0x02 => debug!(" GPIOFS10"),
                0x12 => debug!(" GPIOOE10"),
                0x22 => debug!(" GPIOD10"),
                _ => panic!("xram unimplemented GPIO register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        },
        // Keyboard controller
        0x0100 ..= 0x011F => {
            let offset = address - 0x0100;
            debug!(" (KBC 0x{:02X}", offset);
            match offset {
                0x06 => debug!(" KBCSTS"),
                _ => panic!("xram unimplemented KBC register 0x{:02X} (0x{:04X})", offset, address)
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
                0x0F => debug!(" WDTFCR1"),
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
        // Embedded controller
        0x1400 ..= 0x143F | 0xFF00 ..= 0xFF2F => {
            let offset = if address >= 0xFF00 {
                address - 0xFF00
            } else {
                address - 0x1400
            };
            debug!(" (EC 0x{:02X}", offset);
            match offset {
                0x01 => debug!(" ECFV"),
                0x0D => debug!(" CLKCFG"),
                0x1D => debug!(" ECSTS"),
                0x20 => debug!(" ECMISC"),
                _ => panic!("xram unimplemented EC register 0x{:02X} (0x{:04X})", offset, address)
            }
            debug!(")");
        }
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
