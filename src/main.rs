#![feature(ptr_as_ref_unchecked)]
#![feature(abi_x86_interrupt)]

#![no_main]
#![no_std]

extern crate alloc;

mod boot;
mod kernel;
mod helpers;

use uefi::prelude::*;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("panic: {}", info.message());

    loop {}
}

#[entry]
fn main() -> Status {
    if let Err(err) = boot::boot() {
        error!("error: {}", err);
    }

    loop {}
}


