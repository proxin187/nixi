use crate::kernel::drivers::tty::Tty;

use alloc::vec::Vec;
use alloc::boxed::Box;


/// A tty pool enables multiple tty sessions.
pub struct Pool {
    handles: Vec<Box<dyn Tty>>,
    active: Option<usize>,
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            handles: Vec::new(),
            active: None,
        }
    }
}

impl Tty for Pool {
    fn write(&mut self, buf: &[u8]) {
        if let Some(tty) = self.active.and_then(|active| self.handles.get_mut(active)) {
            tty.write(buf);
        }
    }
}


