//! The boot phase initializes drivers and modules before passing control to the kernel entry.

mod error;

use crate::kernel::mem::pma;
use crate::kernel::irq;

use error::BootError;

use uefi::table::cfg::ConfigTableEntry;
use uefi::prelude::*;


pub struct BootInfo {
    pub acpi: *const core::ffi::c_void,
}

pub fn boot() -> Result<(), BootError> {
    let mut acpi: Option<*const core::ffi::c_void> = None;

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

            irq::init();

            pma::init(&mmap);

            Ok(())
        },
        None => Err(BootError::AcpiNotFound),
    }
}


