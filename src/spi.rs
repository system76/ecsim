// SPDX-License-Identifier: MIT

#![allow(clippy::needless_range_loop)]

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
    pub fast_read_addr: Option<usize>,
    pub aai_addr: Option<usize>,
    pub input: VecDeque<u8>,
    pub output: VecDeque<u8>,
}

#[allow(clippy::new_without_default)]
impl Spi {
    pub fn new() -> Self {
        Self {
            write: false,
            fast_read_addr: None,
            aai_addr: None,
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }

    pub fn step(&mut self, flash: &mut [u8], _flash_name: &str) {
        if let Some(command) = self.input.pop_front() {
            debug!("\n[spi {}", _flash_name);

            self.fast_read_addr = None;

            match command {
                0x01 => {
                    debug!(" write status");

                    let _value = self.input.pop_front().expect("spi wrate status value missing");

                    debug!(" 0x{:02X}", _value);
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
                0x04 => {
                    debug!(" write disable");
                    self.write = false;
                    self.aai_addr = None;
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
                0x0B => {
                    debug!(" fast read");

                    let a2 = self.input.pop_front().expect("spi fast read value missing");
                    let a1 = self.input.pop_front().expect("spi fast read value missing");
                    let a0 = self.input.pop_front().expect("spi fast read value missing");
                    let _d = self.input.pop_front().expect("spi fast read value missing");

                    let address = {
                        (a0 as usize) |
                        (a1 as usize) << 8 |
                        (a2 as usize) << 16
                    };

                    debug!(" 0x{:06X}", address);

                    self.fast_read_addr = Some(address);
                },
                0x50 => {
                    debug!(" write volatile status register");
                    // TODO
                },
                0x60 => {
                    debug!(" chip erase");
                    for i in 0..flash.len() {
                        flash[i] = 0xFF;
                    }
                },
                0x9F => {
                    debug!(" jedec id");
                    self.output.push_back(0xEF);
                    self.output.push_back(0xEF);
                    self.output.push_back(0xEF);
                },
                0xAD => {
                    debug!(" aai program");

                    let addr = if self.input.len() > 2 {
                        let a2 = self.input.pop_front().expect("spi aai program value missing");
                        let a1 = self.input.pop_front().expect("spi aai program value missing");
                        let a0 = self.input.pop_front().expect("spi aai program value missing");

                        (a0 as usize) |
                        (a1 as usize) << 8 |
                        (a2 as usize) << 16
                    } else {
                        self.aai_addr.expect("spi aai address not set")
                    };

                    debug!(" 0x{:06X}", addr);

                    let d0 = self.input.pop_front().expect("spi aai program value missing");
                    let d1 = self.input.pop_front().expect("spi aai program value missing");

                    debug!(" = 0x{:02X}, 0x{:02X}", d0, d1);

                    flash[addr] = d0;
                    flash[addr + 1] = d1;

                    self.aai_addr = Some(addr + 2);
                },
                0xD7 => {
                    debug!(" page erase");

                    let a2 = self.input.pop_front().expect("spi page erase value missing");
                    let a1 = self.input.pop_front().expect("spi page erase value missing");
                    let a0 = self.input.pop_front().expect("spi page erase value missing");

                    let addr =
                        (a0 as usize) |
                        (a1 as usize) << 8 |
                        (a2 as usize) << 16;

                    debug!(" 0x{:06X}", addr);

                    for i in addr..addr + 256 {
                        flash[i] = 0xFF;
                    }
                },
                _ => {
                    panic!("unknown SPI command 0x{:02X}", command);
                }
            }

            assert_eq!(self.input.len(), 0);

            debug!("]");
        }

        if let Some(mut address) = self.fast_read_addr.take() {
            self.output.push_back(flash[address]);
            address += 1;
            if address < flash.len() {
                self.fast_read_addr = Some(address);
            }
        }
    }
}
