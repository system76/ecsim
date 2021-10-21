// SPDX-License-Identifier: MIT

use area8051::{Addr, Isa, Mem};
use std::{env, fs, io};
use std::collections::{BTreeMap, HashMap};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};

pub (crate) mod cmd;

pub use self::ec::Ec;
mod ec;

use self::socket::socket_op;
mod socket;

pub use self::spi::Spi;
mod spi;

pub use self::xram::xram;
mod xram;

type CommandMap = HashMap<&'static str, Box<dyn Fn(&mut Ec, &[&str])>>;

struct Completer<'a> {
    commands: &'a CommandMap
}

impl<'a> liner::Completer for Completer<'a> {
    fn completions(&mut self, start: &str) -> Vec<String> {
        let mut completions = Vec::new();
        for (name, _func) in self.commands {
            if name.starts_with(start) {
                completions.push(name.to_string());
            }
        }
        completions
    }
}

static QUIT: AtomicBool = AtomicBool::new(false);
static RUNNING: AtomicBool = AtomicBool::new(true);
static STEP: AtomicBool = AtomicBool::new(false);

fn commands() -> CommandMap {
    let mut commands: CommandMap = HashMap::new();
    let mut command_help = BTreeMap::new();

    macro_rules! command {
        ($name: expr, $help: expr, $func: expr) => ({
            commands.insert($name, Box::new($func));
            command_help.insert($name, $help);
        });
    }

    command!("continue", "continue execution", |_, _| {
        eprintln!("continuing...");
        RUNNING.store(true, Ordering::SeqCst);
    });
    command!("echo", "print arguments", |_, args: &[&str]| {
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                eprint!(" ");
            }
            eprint!("{}", arg);
        }
        eprintln!();
    });
    command!("quit", "quit program", |_, _| {
        eprintln!("quiting...");
        QUIT.store(true, Ordering::SeqCst);
    });
    command!("step", "execute one instruction", |ec: &mut Ec, _| {
        let mcu = ec.mcu.lock().unwrap();
        eprintln!("step: {:04X}", mcu.pc);
        STEP.store(true, Ordering::SeqCst);
    });
    command!("steps", "number of instructions executed", |ec: &mut Ec, _| {
        eprintln!("steps: {}", ec.steps);
    });

    command!("iram", "dump internal RAM", |ec: &mut Ec, _| {
        let mcu = ec.mcu.lock().unwrap();
        eprintln!("iram:");
        for row in 0..mcu.iram.len() / 16 {
            let row_offset = row * 16;
            eprint!("{:04X}:", row_offset);
            for col in 0..16 {
                eprint!(" {:02X}", mcu.iram[row_offset + col]);
            }
            eprintln!();
        }
    });
    command!("pc", "show program counter", |ec: &mut Ec, _| {
        let mcu = ec.mcu.lock().unwrap();
        eprintln!("pc: {:04X}", mcu.pc);
    });
    command!("xram", "dump external RAM", |ec: &mut Ec, args: &[&str]| {
        let mut mcu = ec.mcu.lock().unwrap();
        if let Some(arg0) = args.get(0) {
            let addr = match u16::from_str_radix(arg0, 16) {
                Ok(ok) => ok as usize,
                Err(err) => {
                    eprintln!("invalid address '{}': {}", arg0, err);
                    return;
                },
            };

            eprintln!("xram {:04X}: {:02X}", addr, mcu.xram[addr]);

            if let Some(arg1) = args.get(1) {
                let value = match u8::from_str_radix(arg1, 16) {
                    Ok(ok) => ok,
                    Err(err) => {
                        eprintln!("invalid value '{}': {}", arg1, err);
                        return;
                    },
                };

                mcu.xram[addr] = value;
                
                eprintln!("xram {:04X}: {:02X}", addr, mcu.xram[addr]);
            }
        } else {
            eprintln!("xram:");
            for row in 0..mcu.xram.len() / 16 {
                let row_offset = row * 16;
                eprint!("{:04X}:", row_offset);
                for col in 0..16 {
                    eprint!(" {:02X}", mcu.xram[row_offset + col]);
                }
                eprintln!();
            }
        }
    });

    command!("int", "trigger interrupt (one argument from 0 to 5)", cmd::int::int);

    command!("kbc_cmd", "send kbc command (one argument in hex)", cmd::kbc::cmd);
    command!("kbc_keyboard", "read kbc keyboard data (as hex)", cmd::kbc::keyboard);
    command!("kbc_mouse", "read kbc mouse data (as hex)", cmd::kbc::mouse);
    command!("kbc_write", "send kbc data (one argument in hex)", cmd::kbc::write);

    command!("pmc_cmd", "send pmc command (one argument in hex)", cmd::pmc::cmd);
    command!("pmc_read", "read pmc data (as hex)", cmd::pmc::read);
    command!("pmc_write", "send pmc data (one argument in hex)", cmd::pmc::write);

    command_help.insert("help", "show command information");
    commands.insert("help", Box::new(move |_, args: &[&str]| {
        for (name, help) in &command_help {
            if args.is_empty() || args.contains(name) {
                eprintln!("  - {} - {}", name, help);
            }
        }
    }));

    commands
}

fn timers(ec: &mut Ec) {
    // Timer information from https://openlabpro.com/guide/timers-8051/
    let mut tcon = ec.load(Addr::Reg(0x88));
    let tmod = ec.load(Addr::Reg(0x89));

    // Timer 0 running
    if tcon & 1 << 4 != 0 {
        if tmod & 0x0F != 0x01 {
            panic!("unimplemented TMOD 0x{:02X}", tmod);
        }

        if ec.steps % 12 == 0 {
            let tl = 0x8A;
            let th = 0x8C;

            let mut count =
                ec.load(Addr::Reg(tl)) as u16 |
                (ec.load(Addr::Reg(th)) as u16) << 8;

            count = count.wrapping_add(1);

            if count == 0 {
                tcon |= 1 << 5;
                //TODO: implement timer 0 interrupts
            }

            ec.store(Addr::Reg(tl), count as u8);
            ec.store(Addr::Reg(th), (count >> 8) as u8);
        }
    }

    // Timer 1 running
    if tcon & 1 << 6 != 0 {
        if tmod & 0xF0 != 0x10 {
            panic!("unimplemented TMOD 0x{:02X}", tmod);
        }

        if ec.steps % 12 == 0 {
            let tl = 0x8B;
            let th = 0x8D;

            let mut count =
                ec.load(Addr::Reg(tl)) as u16 |
                (ec.load(Addr::Reg(th)) as u16) << 8;

            count = count.wrapping_add(1);

            if count == 0 {
                tcon |= 1 << 7;
                //TODO: implement timer 1 interrupts
            }

            ec.store(Addr::Reg(tl), count as u8);
            ec.store(Addr::Reg(th), (count >> 8) as u8);
        }
    }

    ec.store(Addr::Reg(0x88), tcon);
}

fn main() {
    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::SeqCst);
    }).expect("failed to set ctrl-c handler");

    let pmem_path = env::args().nth(1).unwrap_or("ec.rom".to_string());

    let mut pmem = fs::read(&pmem_path).expect("failed to read ec.rom");

    // Fill program memory to 128 KiB
    while pmem.len() < 128 * 1024 {
        pmem.push(0xFF);
    }

    let xmem = pmem.clone();

    let mut ec = Ec::new(
        0x5570, 0x01, // IT5570 (B Version)
        //0x8587, 0x06, // IT8587E/VG (F Version)
        pmem.into_boxed_slice(),
        xmem.into_boxed_slice()
    );

    ec.reset();

    let commands = commands();

    let mut socket_opt = UdpSocket::bind("127.0.0.1:8587").ok();
    if let Some(ref mut socket) = socket_opt {
        socket.set_nonblocking(true).expect("failed to set socket nonblocking");
    }

    let mut con = liner::Context::new();
    while ! QUIT.load(Ordering::SeqCst) {
        while STEP.swap(false, Ordering::SeqCst) || RUNNING.load(Ordering::SeqCst) {
            if let Some(ref mut socket) = socket_opt {
                let mut request = [0x00; 4];
                match socket.recv_from(&mut request) {
                    Ok((count, addr)) => if count >= request.len() {
                        let response = socket_op(&mut ec, &request);
                        socket.send_to(&response, addr).expect("failed to write socket");
                    },
                    Err(err) => match err.kind() {
                        io::ErrorKind::WouldBlock => (),
                        io::ErrorKind::Interrupted => {
                            eprintln!("^C");
                        },
                        io::ErrorKind::UnexpectedEof => {
                            eprintln!("^D");
                            QUIT.store(true, Ordering::SeqCst);
                        },
                        _ => {
                            panic!("failed to read socket: {:?}", err);
                        }
                    }
                }
            }

            ec.step();

            // Check pcon for idle or power down
            let pcon = ec.load(Addr::Reg(0x87));
            if (pcon & 0b11) != 0 {
                //panic!("unimplemented PCON 0x{:02X}", pcon);
            }

            // Serial bus
            let s = ec.load(Addr::Reg(0x98));
            let b = ec.load(Addr::Reg(0x99));
            if b > 0 {
                print!("{}", b as char);
                ec.store(Addr::Reg(0x98), s | (1 << 1));
                ec.store(Addr::Reg(0x99), 0);
            }


            timers(&mut ec);

            // if ec.steps % 1_000_000 == 0 {
            //     println!("{}M steps", ec.steps / 1_000_000);
            // }

            if ec.pc() == 0 {
                eprintln!("reset!");
                //RUNNING.store(false, Ordering::SeqCst);
            }

            ec.steps += 1;
        }

        match con.read_line(
            "[ecsim]$ ",
            None,
            &mut Completer {
                commands: &commands,
            }
        ) {
            Ok(ok) => {
                let mut parts = ok.split(' ').filter(|x| ! x.is_empty());
                if let Some(command) = parts.next() {
                    if let Some(func) = commands.get(command) {
                        let args: Vec<&str> = parts.collect();
                        func(&mut ec, &args);
                    } else {
                        eprintln!("unknown command: {}", ok);
                    }

                    con.history.push(ok.into()).unwrap();
                }
            },
            Err(err) => match err.kind() {
                io::ErrorKind::Interrupted => {
                    eprintln!("^C");
                },
                io::ErrorKind::UnexpectedEof => {
                    eprintln!("^D");
                    QUIT.store(true, Ordering::SeqCst);
                },
                _ => {
                    panic!("error: {:?}", err);
                }
            }
        }
    }
}
