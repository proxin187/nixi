use crate::kernel::drivers::tty::Tty;

use x86_64::instructions::interrupts;
use x86::io;


/// A serial port tty. It acts as a passthrough layer, forwarding data directly to the serial port.
pub struct Serial {
    port: u16,
}

impl Tty for Serial {
    fn write(&mut self, buf: &[u8]) {
        interrupts::without_interrupts(|| {
            for byte in buf {
                unsafe {
                    while io::inb(self.port + 5) & 0x20 == 0 {}

                    io::outb(self.port, *byte);
                }
            }
        });
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


