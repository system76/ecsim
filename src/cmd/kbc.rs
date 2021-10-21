// SPDX-License-Identifier: MIT

use crate::Ec;

const STATUS_CMD: u8 = 1 << 3;
const STATUS_IBF: u8 = 1 << 1;
const STATUS_OBF: u8 = 1 << 0;

const STATUS: usize = 0x1304;
const DATA_KEYBOARD: usize = 0x1506;
const DATA_MOUSE: usize = 0x1508;
const DATA_IN: usize = 0x130A;

pub fn cmd(ec: &mut Ec, args: &[&str]) {
    if args.len() != 1 {
        eprintln!("kbc_cmd [argument in hex]");
        return;
    }

    let data = match u8::from_str_radix(args[0], 16) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("argument '{}' failed to parse as hex: {}", args[0], err);
            eprintln!("kbc_cmd [argument in hex]");
            return;
        }
    };

    let mut mcu = ec.mcu.lock().unwrap();
    mcu.xram[STATUS] |= STATUS_CMD | STATUS_IBF;
    mcu.xram[DATA_IN] = data;
}

pub fn keyboard(ec: &mut Ec, _args: &[&str]) {
    let mut mcu = ec.mcu.lock().unwrap();
    //TODO: Determine if byte came from keyboard
    if mcu.xram[STATUS] & STATUS_OBF != 0 {
        eprintln!("{:02X}", mcu.xram[DATA_KEYBOARD]);
        mcu.xram[STATUS] &= !STATUS_OBF;
    }
}

pub fn mouse(ec: &mut Ec, _args: &[&str]) {
    let mut mcu = ec.mcu.lock().unwrap();
    //TODO: Determine if byte came from mouse
    if mcu.xram[STATUS] & STATUS_OBF != 0 {
        eprintln!("{:02X}", mcu.xram[DATA_MOUSE]);
        mcu.xram[STATUS] &= !STATUS_OBF;
    }
}

pub fn write(ec: &mut Ec, args: &[&str]) {
    if args.len() != 1 {
        eprintln!("kbc_write [hex argument]");
        return;
    }

    let data = match u8::from_str_radix(args[0], 16) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("argument '{}' failed to parse as hex: {}", args[0], err);
            eprintln!("kbc_write [hex argument]");
            return;
        }
    };

    let mut mcu = ec.mcu.lock().unwrap();
    mcu.xram[STATUS] &= !STATUS_CMD;
    mcu.xram[STATUS] |= STATUS_IBF;
    mcu.xram[DATA_IN] = data;
}
