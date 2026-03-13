//! The boot phase initializes subsystems before passing control to the kernel entry

mod error;

use crate::kernel::mem::pma;
use crate::kernel::irq;
use crate::kernel;

use error::BootError;

use uefi::table::cfg::ConfigTableEntry;
use uefi::prelude::*;


/// Exit boot services, initialize all subsystems and jump to kernel entry
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

            kernel::entry();
        },
        None => Err(BootError::AcpiNotFound),
    }
}


