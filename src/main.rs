#![feature(ptr_as_ref_unchecked)]
#![feature(once_cell_try_insert)]
#![feature(iter_next_chunk)]
#![feature(iter_map_windows)]
#![feature(naked_functions_rustic_abi)]
#![feature(abi_x86_interrupt)]
#![feature(arbitrary_self_types_pointers)]
#![feature(option_reference_flattening)]
#![feature(never_type)]
#![no_main]
#![no_std]

extern crate alloc;

mod arch;
mod boot;
mod device;
mod drivers;
mod fs;
mod helpers;
mod irq;
mod loader;
mod mem;
mod parse;
mod scheduler;
mod syscall;
mod vfs;

use core::sync::atomic::Ordering;

use arch::x86_64::registers;
use mem::paging;

use uefi::prelude::*;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "panic in '{}' at line {}: {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        error!("panic: {}", info.message());
    }

    loop {}
}

/// Exit boot services, initialize all subsystems and load init process
#[entry]
pub fn main() -> Status {
    paging::KERNEL_PAGE_TABLE.store(registers::read_cr3(), Ordering::Relaxed);

    boot::boot();
}
