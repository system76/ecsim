// SPDX-License-Identifier: MIT

use std::io;
use std::net::UdpSocket;

static mut SOCKET: Option<UdpSocket> = None;

fn transaction(kind: u8, addr: u16, value: u8) -> io::Result<u8> {
    let socket = unsafe { SOCKET.as_ref().expect("SOCKET not initialized") };

    let addr = addr.to_le_bytes();
    let request = [kind as u8, addr[0], addr[1], value];
    if socket.send(&request)? != request.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Socket request incorrect size"
        ));
    }

    let mut response = [0];
    if socket.recv(&mut response)? != response.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Socket response incorrect size"
        ));
    }

    Ok(response[0])
}

pub fn init() -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:8587")?;
    unsafe { SOCKET = Some(socket) };

    transaction(0x00, 0, 0)?;
    Ok(())
}

pub fn inb(addr: u16) -> io::Result<u8> {
    transaction(0x01, addr, 0)
}

pub fn outb(addr: u16, value: u8) -> io::Result<()> {
    transaction(0x02, addr, value)?;
    Ok(())
}
