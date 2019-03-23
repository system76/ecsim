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
    debug!(" [xram 0x{:02X}", address);

    let old = mcu.load(Addr::XRam(address));

    match address {
        // Scratch SRAM
        0x0000 ... 0x0FFF => {
            debug!(" (SRAM)");
        },
        // SMFI
        0x1000 ... 0x107F => {
            let address = address - 0x1000;
            debug!(" (SMFI 0x{:02X})", address);
            match address {
                0x01 => {
                },
                _ => panic!("xram unimplemented SMFI register 0x{:02X}", address)
            }
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
