use crate::ec::Ec;

#[cfg(feature = "debug_socket")]
macro_rules! debug {
    ($($arg:tt)*) => (eprint!($($arg)*));
}

#[cfg(not(feature = "debug_socket"))]
macro_rules! debug {
    ($($arg:tt)*) => (());
}

pub fn socket_op(ec: &mut Ec, request: &[u8; 3]) -> [u8; 1] {
    debug!("\n[socket");

    let mut response = [0x00];
    match request[0] {
        0x00 => {
            debug!(" init");
        },
        0x01 => {
            let port = request[1];
            let mut value = 0;
            debug!(" read 0x{:02X}", port);
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
                    let mut mcu = ec.mcu.lock().unwrap();
                    mcu.xram[0x1500] &= !(1 << 0);
                    value = mcu.xram[0x1501];
                },
                0x66 => {
                    debug!(" (pmc status)");
                    let mcu = ec.mcu.lock().unwrap();
                    value = mcu.xram[0x1500];
                },
                _ => {
                    debug!(" (unimplemented)");
                }
            }
            debug!(" = 0x{:02X}", value);
            response[0] = value;
        },
        0x02 => {
            let port = request[1];
            let value = request[2];
            debug!(" write 0x{:02X}, 0x{:02X}", port, value);
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
                    let mut mcu = ec.mcu.lock().unwrap();
                    mcu.xram[0x1500] &= !(1 << 3);
                    mcu.xram[0x1500] |= 1 << 1;
                    mcu.xram[0x1504] = value;
                },
                0x66 => {
                    debug!(" (pmc command)");
                    let mut mcu = ec.mcu.lock().unwrap();
                    mcu.xram[0x1500] |= (1 << 3) | (1 << 1);
                    mcu.xram[0x1504] = value;
                },
                _ => {
                    debug!(" (unimplemented)");
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
