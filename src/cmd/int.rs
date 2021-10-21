// SPDX-License-Identifier: MIT

use area8051::Isa;

use crate::Ec;

pub fn int(ec: &mut Ec, args: &[&str]) {
    if args.len() != 1 {
        eprintln!("int [argument from 0 to 5]");
        return;
    }

    let int = match u8::from_str_radix(&args[0], 10) {
        Ok(ok) => if ok <= 5 {
            ok
        } else {
            eprintln!("argument '{}' greater than 5", args[0]);
            eprintln!("int [argument from 0 to 5]");
            return;
        },
        Err(err) => {
            eprintln!("argument '{}' failed to parse: {}", args[0], err);
            eprintln!("int [argument from 0 to 5]");
            return;
        }
    };

    let mut mcu = ec.mcu.lock().unwrap();
    let pc = mcu.pc();
    mcu.push_sp(pc as u8);
    mcu.push_sp((pc >> 8) as u8);
    mcu.set_pc(0x0003 + (int as u16) * 8);
}
