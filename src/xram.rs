use area8051::{Rom, Ram};
use std::sync::Mutex;

#[cfg(feature = "debug_xram")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_xram"))]
macro_rules! debug {
    ($($arg:tt)*) => ();
}

pub struct XRam {
    data: Mutex<Vec<u8>>
}

impl XRam {
    pub fn new() -> XRam {
        let mut data = vec![0; 65536];

        // FPCFG
        data[0x1001] = 0b1011_1111;

        Self {
            data: Mutex::new(data)
        }
    }

    fn transaction(&self, address: u16, new_opt: Option<u8>) -> u8 {
        let mut data = self.data.lock().unwrap();

        debug!(" [xram 0x{:02X}", address);

        let old = data.load(address);

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
            data.store(address, new);
        }

        debug!("]");

        old
    }
}

impl Rom for XRam {
    fn load(&self, address: u16) -> u8 {
        self.transaction(address, None)
    }
}

impl Ram for XRam {
    fn store(&mut self, address: u16, value: u8) {
        self.transaction(address, Some(value));
    }
}
