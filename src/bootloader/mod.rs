mod error;

use crate::kernel::mem::pma;

use error::BootError;

use uefi::table::cfg::ConfigTableEntry;
use uefi::mem::memory_map::MemoryMapOwned;
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

            pma::init(mmap);

            Ok(())
        },
        None => Err(BootError::AcpiNotFound),
    }
}


