#![feature(ptr_as_ref_unchecked)]
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

use arch::x86_64::{self, tables};
use loader::Loader;
use loader::error::LoaderError;
use mem::pma;
use scheduler::context;
use vfs::MountSource;

use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::cfg::ConfigTableEntry;

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

/// Load the init process
pub fn load_init() -> Result<!, LoaderError> {
    vfs::with_vfs(|vfs| {
        let root = vfs.root();

        let _ = vfs.mount(
            root,
            MountSource::FileSystem {
                name: "initramfs",
                device: None,
            },
        );
    });

    Loader::from_fs("/init")?.load()?;

    context::enter_usermode();
}

/// Exit boot services, initialize all subsystems and load init process
#[entry]
pub fn main() -> Status {
    let mut acpi: Option<*const core::ffi::c_void> = None;

    let handle = boot::image_handle();
    let image = boot::open_protocol_exclusive::<LoadedImage>(handle)
        .expect("failed to open loaded image protocol");

    let (base, _) = image.info();

    log!("kernel loaded at: {:#x?}", base);

    system::with_config_table(|table| {
        for entry in table {
            if entry.guid == ConfigTableEntry::ACPI2_GUID {
                acpi = Some(entry.address);
            }
        }
    });

    match acpi {
        Some(acpi) => {
            let mmap = unsafe { boot::exit_boot_services(None) };

            x86_64::init();

            tables::init();

            irq::init();

            pma::init(&mmap);

            let err = load_init().unwrap_err();

            panic!("failed to load init: {}", err);
        }
        None => panic!("ACPI not found"),
    }
}
