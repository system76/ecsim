use std::collections::VecDeque;

#[cfg(feature = "debug_spi")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_spi"))]
macro_rules! debug {
    ($($arg:tt)*) => (());
}

pub struct Spi {
    pub write: bool,
    pub input: VecDeque<u8>,
    pub output: VecDeque<u8>,
}

impl Spi {
    pub fn new() -> Self {
        Self {
            write: false,
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }

    pub fn step(&mut self, flash: &mut [u8]) {
        if let Some(command) = self.input.pop_front() {
            debug!("\n[spi");

            match command {
                0x01 => {
                    debug!(" write status");

                    let value = self.input.pop_front().expect("spi wrate status value missing");

                    debug!(" 0x{:02X}", value);
                },
                0x02 => {
                    debug!(" page program");

                    let a2 = self.input.pop_front().expect("spi page program value missing");
                    let a1 = self.input.pop_front().expect("spi page program value missing");
                    let a0 = self.input.pop_front().expect("spi page program value missing");

                    let mut address = {
                        (a0 as usize) |
                        (a1 as usize) << 8 |
                        (a2 as usize) << 16
                    };

                    debug!(" 0x{:06X}", address);

                    while let Some(value) = self.input.pop_front() {
                        debug!(" [0x{:06X} = 0x{:02X}]", address, value);

                        flash[address] = value;

                        if address & 0xFF == 0xFF {
                            address -= 0xFF;
                        } else {
                            address += 1;
                        }
                    }
                },
                0x05 => {
                    debug!(" read status");
                    let value = (self.write as u8) << 1;
                    self.output.push_back(value);
                },
                0x06 => {
                    debug!(" write enable");
                    self.write = true;
                },
                _ => {
                    panic!("unknown SPI command 0x{:02X}", command);
                }
            }

            assert!(self.input.is_empty());

            debug!("]");
        }
    }
}
