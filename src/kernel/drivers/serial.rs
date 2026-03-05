use core::fmt::Write;

use x86_64::instructions::interrupts;
use x86::io;
use spin::Mutex;
use lazy_static::lazy_static;


lazy_static! {
    static ref SERIAL: Mutex<Serial> = Mutex::new(Serial::new(0x3f8));
}

#[inline]
pub fn write_fmt(args: core::fmt::Arguments) {
    let _ = SERIAL.lock().write_fmt(args);
}

pub struct Serial {
    port: u16,
}

impl Write for Serial {
    fn write_str(&mut self, string: &str) -> Result<(), core::fmt::Error> {
        interrupts::without_interrupts(|| {
            for byte in string.bytes() {
                unsafe {
                    while io::inb(self.port + 5) & 0x20 == 0 {}

                    io::outb(self.port, byte);
                }
            }
        });

        Ok(())
    }
}

impl Serial {
    pub fn new(port: u16) -> Serial {
        unsafe {
            io::outb(port + 1, 0x00);
            io::outb(port + 3, 0x80);
            io::outb(port, 0x03);
            io::outb(port + 1, 0x00);
            io::outb(port + 3, 0x03);
            io::outb(port + 2, 0xc7);
            io::outb(port + 4, 0x0b);
            io::outb(port + 4, 0x0f);
        }

        Serial {
            port,
        }
    }
}


