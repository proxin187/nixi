#![feature(ptr_as_ref_unchecked)]

#![no_main]
#![no_std]

mod bootloader;
mod kernel;
mod helpers;

use uefi::prelude::*;
use uefi::println;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("error: bootd panic: {}", info.message());

    loop {}
}

#[entry]
fn main() -> Status {
    if let Err(err) = bootloader::boot() {
        println!("bootloader: error: {}", err);
    }

    loop {}
}


