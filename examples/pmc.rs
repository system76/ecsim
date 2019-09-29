/// Example of resetting KBC - communicates with running ecsim over socket

use std::io;

// TODO: allow using either socket or real hardware
use support::{init, inb, outb};
mod support;

const STATUS_IBF: u8 = 1 << 1;
const STATUS_OBF: u8 = 1 << 0;

const PMC_STATUS: u8 = 0x66;
const PMC_DATA: u8 = 0x62;

fn pmc_cmd(cmd: u8) -> io::Result<()> {
    //TODO: timeout
    while inb(PMC_STATUS)? & STATUS_IBF != 0 {}
    outb(PMC_STATUS, cmd)
}

fn pmc_read() -> io::Result<u8> {
    //TODO: timeout
    while inb(PMC_STATUS)? & STATUS_OBF == 0 {}
    inb(PMC_DATA)
}

fn pmc_write(data: u8) -> io::Result<()> {
    //TODO: timeout
    while inb(PMC_STATUS)? & STATUS_IBF != 0 {}
    outb(PMC_DATA, data)
}

fn acpi_read(addr: u8) -> io::Result<u8> {
    pmc_cmd(0x80)?;
    pmc_write(addr)?;
    pmc_read()
}

fn acpi_write(addr: u8, data: u8) -> io::Result<()> {
    pmc_cmd(0x81)?;
    pmc_write(addr)?;
    pmc_write(data)
}

fn main() -> io::Result<()> {
    init()?;

    println!("AC connected: {}", acpi_read(0x10)? & 1 != 0);

    Ok(())
}
