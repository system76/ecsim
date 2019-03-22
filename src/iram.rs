use area8051::{Rom, Ram};
use std::sync::Mutex;

#[cfg(feature = "debug_iram")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_iram"))]
macro_rules! debug {
    ($($arg:tt)*) => ();
}

pub struct IRam {
    data: Mutex<Vec<u8>>
}

impl IRam {
    pub fn new() -> IRam {
        let mut data = vec![0; 256];

        // Set port 0 to 0xFF
        data[0x80] = 0xFF;
        // Set stack pointer to 0x07
        data[0x81] = 0x07;
        // Set port 1 to 0xFF
        data[0x90] = 0xFF;
        // Set port 2 to 0xFF
        data[0xA0] = 0xFF;
        // Set port 3 to 0xFF
        data[0xB0] = 0xFF;

        Self {
            data: Mutex::new(data)
        }
    }

    fn transaction(&self, address: u16, new_opt: Option<u8>) -> u8 {
        let mut data = self.data.lock().unwrap();

        debug!(" [iram 0x{:02X}", address);

        let old = data.load(address);

        match address {
            // General purpose registers
            0x00 ... 0x07 => {
                debug!(" (R{}B0)", address);
            },
            0x08 ... 0x0F => {
                debug!(" (R{}B1)", address - 0x08);
            },
            0x10 ... 0x17 => {
                debug!(" (R{}B2)", address - 0x10);
            },
            0x18 ... 0x1F => {
                debug!(" (R{}B3)", address - 0x18);
            },
            // Port 0
            0x80 => {
                debug!(" (P0)");
            },
            // Stack pointer
            0x81 => {
                debug!(" (SP)");
            },
            // Data pointer low
            0x82 => {
                debug!(" (DPL)");
            },
            // Data pointer high
            0x83 => {
                debug!(" (DPH)");
            },
            // Data pointer low 1
            0x84 => {
                debug!(" (DPL1)");
            },
            // Data pointer high 1
            0x85 => {
                debug!(" (DPH1)");
            },
            // Data pointer select
            0x86 => {
                debug!(" (DPS)");
            },
            // Power control
            0x87 => {
                debug!(" (PCON)");
            },
            // Timer control
            0x88 => {
                debug!(" (TCON)");
            },
            // Timer mode
            0x89 => {
                debug!(" (TMOD)");
            },
            // Serial control
            0x98 => {
                debug!(" (SCON)");
            },
            // Interrupt enable
            0xA8 => {
                debug!(" (IE)");
            },
            // Program status word
            0xD0 => {
                debug!(" (PSW)");
            },
            // Accumulator
            0xE0 => {
                debug!(" (A)");
            },
            // B
            0xF0 => {
                debug!(" (B)");
            },
            // Unknown
            _ => (),
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

impl Rom for IRam {
    fn load(&self, address: u16) -> u8 {
        self.transaction(address, None)
    }
}

impl Ram for IRam {
    fn store(&mut self, address: u16, value: u8) {
        self.transaction(address, Some(value));
    }
}
