use area8051::{Addr, Mcu, Mem};

#[cfg(feature = "debug_xram")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_xram"))]
macro_rules! debug {
    ($($arg:tt)*) => ();
}

pub fn xram(mcu: &mut Mcu, address: u16, new_opt: Option<u8>) -> u8 {
    debug!(" [xram 0x{:04X}", address);

    let mut old = mcu.load(Addr::XRam(address));

    match address {
        // Scratch SRAM
        0x0000 ... 0x0FFF => {
            debug!(" (SRAM)");
        },
        // SMFI
        0x1000 ... 0x107F => {
            let address = address - 0x1000;
            debug!(" (SMFI 0x{:02X}", address);
            match address {
                0x00 => debug!(" FBCFG"),
                0x01 => debug!(" FPCFG"),
                0x07 => debug!(" UNKNOWN"),
                0x3B => debug!(" ECINDAR0"),
                0x3C => debug!(" ECINDAR1"),
                0x3D => debug!(" ECINDAR2"),
                0x3E => debug!(" ECINDAR3"),
                0x3F => {
                    debug!(" ECINDDR");
                    let ecindar = {
                        (mcu.load(Addr::XRam(0x103B)) as u32) |
                        (mcu.load(Addr::XRam(0x103C)) as u32) << 8 |
                        (mcu.load(Addr::XRam(0x103D)) as u32) << 16 |
                        (mcu.load(Addr::XRam(0x103E)) as u32) << 24
                    };

                    debug!(" [flash address 0x{:08X}]", ecindar);
                    old = mcu.pmem[ecindar as usize];
                    if let Some(new) = new_opt {
                        mcu.pmem[ecindar as usize] = new;
                    }
                },
                0x58 => debug!(" HINSTC1"),
                0x63 => debug!(" FLHCTRL3R"),
                _ => panic!("xram unimplemented SMFI register 0x{:02X}", address)
            }
            debug!(")");
        },
        0x2200 ... 0x22FF => {
            let address = address - 0x2200;
            debug!(" (BRAM 0x{:02X}", address);
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
