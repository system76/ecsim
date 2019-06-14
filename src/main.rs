use area8051::{Addr, Isa, Mem};
use std::{fs, io};
use std::sync::atomic::{AtomicBool, Ordering};

pub use self::ec::Ec;
mod ec;

pub use self::spi::Spi;
mod spi;

pub use self::xram::xram;
mod xram;

struct Completer;

impl liner::Completer for Completer {
    fn completions(&mut self, _start: &str) -> Vec<String> {
        Vec::new()
    }
}

static RUNNING: AtomicBool = AtomicBool::new(true);

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

    let mut step = false;
    let mut con = liner::Context::new();
    loop {
        while step || RUNNING.load(Ordering::SeqCst) {
            step = false;
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

        match con.read_line("[ecsim]$ ", None, &mut Completer) {
            Ok(ok) => {
                let mcu = ec.mcu.lock().unwrap();

                match ok.as_str() {
                    "continue" => {
                        eprintln!("continuing...");
                        RUNNING.store(true, Ordering::SeqCst);
                    },
                    "quit" => {
                        break;
                    },
                    "step" => {
                        eprintln!("step: {:04X}", mcu.pc);
                        step = true;
                    },
                    "" => (),

                    "pc" => {
                        eprintln!("pc: {:04X}", mcu.pc);
                    },
                    "iram" => {
                        eprintln!("xram:");
                        for row in 0..mcu.iram.len() / 16 {
                            let row_offset = row * 16;
                            eprint!("{:04X}:", row_offset);
                            for col in 0..16 {
                                eprint!(" {:02X}", mcu.iram[row_offset + col]);
                            }
                            eprintln!();
                        }
                    },
                    "xram" => {
                        eprintln!("xram:");
                        for row in 0..mcu.xram.len() / 16 {
                            let row_offset = row * 16;
                            eprint!("{:04X}:", row_offset);
                            for col in 0..16 {
                                eprint!(" {:02X}", mcu.xram[row_offset + col]);
                            }
                            eprintln!();
                        }
                    },

                    unknown => {
                        eprintln!("unknown command: {}", unknown);
                    }
                }

                con.history.push(ok.into()).unwrap();
            },
            Err(err) => match err.kind() {
                io::ErrorKind::Interrupted => {
                    eprintln!("^C");
                },
                io::ErrorKind::UnexpectedEof => {
                    eprintln!("^D");
                    break;
                },
                _ => {
                    panic!("error: {:?}", err);
                }
            }
        }
    }
}
