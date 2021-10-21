// SPDX-License-Identifier: MIT

use crate::ec::Ec;

#[cfg(feature = "debug_socket")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_socket"))]
macro_rules! debug {
    ($($arg:tt)*) => (());
}

pub fn socket_op(ec: &mut Ec, request: &[u8; 4]) -> [u8; 1] {
    debug!("\n[socket");

    let mut mcu = ec.mcu.lock().unwrap();

    let hramwc = mcu.xram[0x105A];
    let hramw0ba = mcu.xram[0x105B];
    let hramw1ba = mcu.xram[0x105C];
    let hramw0aas = mcu.xram[0x105D];
    let hramw1aas = mcu.xram[0x105E];

    // TODO: protection bits
    let hramw0_length = 2u16.pow(4 + (hramw0aas as u32 & 0x7));
    let hramw1_length = 2u16.pow(4 + (hramw1aas as u32 & 0x7));

    let hramw0_start = (hramw0ba as u16) << 4;
    let hramw1_start = (hramw1ba as u16) << 4;
    let hramw0_end = hramw0_start + hramw0_length - 1;
    let hramw1_end = hramw1_start + hramw1_length - 1;

    let mut response = [0x00];
    match request[0] {
        // init
        0x00 => {
            debug!(" init");
        },
        // inb
        0x01 => {
            let port = u16::from_le_bytes([request[1], request[2]]);
            let mut value = 0;
            debug!(" read 0x{:04X}", port);
            match port {
                0x2e => {
                    debug!(" (super io address)");
                    value = ec.superio_addr;
                },
                0x2f => {
                    debug!(" (super io data 0x{:02X})", ec.superio_addr);
                    match ec.superio_addr {
                        0x20 => {
                            debug!(" (EC ID high)");
                            value = (ec.id >> 8) as u8;
                        },
                        0x21 => {
                            debug!(" (EC ID low)");
                            value = ec.id as u8;
                        },
                        _ => {
                            debug!(" (unimplemented)");
                        }
                    }
                },
                0x62 => {
                    debug!(" (pmc data)");
                    mcu.xram[0x1500] &= !(1 << 0);
                    value = mcu.xram[0x1501];
                },
                0x66 => {
                    debug!(" (pmc status)");
                    value = mcu.xram[0x1500];
                },
                _ => {
                    if ((hramw0_start ..= hramw0_end).contains(&port) && (hramwc & 0b01 != 0)) ||
                       ((hramw1_start ..= hramw1_end).contains(&port) && (hramwc & 0b10 != 0)) {
                        debug!(" (h2ram)");
                        value = mcu.xram[port as usize];
                    } else {
                        debug!(" (unimplemented)");
                    }
                }
            }
            debug!(" = 0x{:02X}", value);
            response[0] = value;
        },
        // outb
        0x02 => {
            let port = u16::from_le_bytes([request[1], request[2]]);
            let value = request[3];
            debug!(" write 0x{:04X}, 0x{:02X}", port, value);
            match port {
                0x2e => {
                    debug!(" (super io address)");
                    ec.superio_addr = value;
                },
                0x2f => {
                    debug!(" (super io data 0x{:02X})", ec.superio_addr);
                    match ec.superio_addr {
                        _ => {
                            debug!(" (unimplemented)");
                        }
                    }
                },
                0x62 => {
                    debug!(" (pmc data)");
                    mcu.xram[0x1500] &= !(1 << 3);
                    mcu.xram[0x1500] |= 1 << 1;
                    mcu.xram[0x1504] = value;
                },
                0x66 => {
                    debug!(" (pmc command)");
                    mcu.xram[0x1500] |= (1 << 3) | (1 << 1);
                    mcu.xram[0x1504] = value;
                },
                _ => {
                    if ((hramw0_start ..= hramw0_end).contains(&port) && (hramwc & 0b01 != 0)) ||
                       ((hramw1_start ..= hramw1_end).contains(&port) && (hramwc & 0b10 != 0)) {
                        debug!(" (h2ram)");
                        mcu.xram[port as usize] = value;
                    } else {
                        debug!(" (unimplemented)");
                    }
                }
            }
            debug!(" = 0x{:02X}", value);
            response[0] = value;
        },
        _ => {
            debug!(" unknown");
        }
    }
    debug!("]");
    response
}
