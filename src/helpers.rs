

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            let _ = crate::kernel::drivers::tty::TTY.lock().write_fmt(format_args!("[{}] {}\n", module_path!(), format_args!($($arg)*)));
        }
    };
}

pub(crate) use log;


