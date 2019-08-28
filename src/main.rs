use area8051::{Addr, Isa, Mem};
use std::{fs, io};
use std::collections::{BTreeMap, HashMap};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};

pub use self::ec::Ec;
mod ec;

use self::socket::socket_op;
mod socket;

pub use self::spi::Spi;
mod spi;

pub use self::xram::xram;
mod xram;

type CommandMap = HashMap<&'static str, Box<Fn(&mut Ec)>>;

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

    command!("continue", "continue execution", |_| {
        eprintln!("continuing...");
        RUNNING.store(true, Ordering::SeqCst);
    });
    command!("quit", "quit program", |_| {
        eprintln!("quiting...");
        QUIT.store(true, Ordering::SeqCst);
    });
    command!("step", "execute one instruction", |ec: &mut Ec| {
        let mcu = ec.mcu.lock().unwrap();
        eprintln!("step: {:04X}", mcu.pc);
        STEP.store(true, Ordering::SeqCst);
    });

    command!("iram", "dump internal RAM", |ec: &mut Ec| {
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
    command!("pc", "show program counter", |ec: &mut Ec| {
        let mcu = ec.mcu.lock().unwrap();
        eprintln!("pc: {:04X}", mcu.pc);
    });
    command!("xram", "dump external RAM", |ec: &mut Ec| {
        let mcu = ec.mcu.lock().unwrap();
        eprintln!("xram:");
        for row in 0..mcu.xram.len() / 16 {
            let row_offset = row * 16;
            eprint!("{:04X}:", row_offset);
            for col in 0..16 {
                eprint!(" {:02X}", mcu.xram[row_offset + col]);
            }
            eprintln!();
        }
    });

    command!("0x9A", "send 0x9A command", |ec: &mut Ec| {
        let mut mcu = ec.mcu.lock().unwrap();
        mcu.xram[0x1500] |= (1 << 3) | (1 << 1);
        mcu.xram[0x1504] = 0x9A;
    });

    command_help.insert("help", "show command information");
    commands.insert("help", Box::new(move |_| {
        for (name, help) in &command_help {
            eprintln!("  - {} - {}", name, help);
        }
    }));

    commands
}

fn main() {
    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::SeqCst);
    }).expect("failed to set ctrl-c handler");

    let pmem = fs::read("ec.rom").expect("failed to read ec.rom");

    let mut ec = Ec::new(
        //0x5570, 0x01, // IT5570 (B Version)
        0x8587, 0x06, // IT8587E/VG (F Version)
        pmem.into_boxed_slice()
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
                let mut request = [0x00; 3];
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
                panic!("unimplemented PCON 0x{:02X}", pcon);
            }

            if ec.pc() == 0 {
                eprintln!("reset!");
                break;
            }
        }

        match con.read_line(
            "[ecsim]$ ",
            None,
            &mut Completer {
                commands: &commands,
            }
        ) {
            Ok(ok) => {
                if let Some(func) = commands.get(ok.as_str()) {
                    func(&mut ec);
                } else if ok.is_empty() {
                    // Ignore empty lines
                } else {
                    eprintln!("unknown command: {}", ok);
                }

                con.history.push(ok.into()).unwrap();
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
