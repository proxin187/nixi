mod serial;
mod pool;

use serial::Serial;

use spin::Mutex;
use lazy_static::lazy_static;

use alloc::boxed::Box;

lazy_static! {
    pub static ref TTY: Mutex<TtyHandle<Serial>> = Mutex::new(TtyHandle::new(Serial::new(0x3f8)));
}

/// A tty is a terminal interface
pub trait Tty {
    fn write(&mut self, buf: &[u8]);
}

impl Tty for Box<dyn Tty> {
    fn write(&mut self, buf: &[u8]) {
        self.as_mut().write(buf);
    }
}

/// A handle to a tty implementation
pub struct TtyHandle<T: Tty> {
    tty: T,
}

impl<T: Tty> TtyHandle<T> {
    pub const fn new(tty: T) -> TtyHandle<T> {
        TtyHandle {
            tty,
        }
    }
}

impl<T: Tty> core::fmt::Write for TtyHandle<T> {
    fn write_str(&mut self, string: &str) -> Result<(), core::fmt::Error> {
        self.tty.write(string.as_bytes());

        Ok(())
    }
}


