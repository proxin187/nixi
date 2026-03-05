mod error;

use crate::kernel::mem::pma::PhysicalMemoryAllocator;
use crate::kernel::mem::paging;

use error::BootError;

use uefi::table::cfg::ConfigTableEntry;
use uefi::prelude::*;


// TODO: add the page table to the boot info, or have it as a static
pub struct BootInfo {
    pub acpi: *const core::ffi::c_void,
    pub pma: PhysicalMemoryAllocator,
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
            // NOTE: we cannot use any boot services after this, it includes println, so we must
            // implement uart
            let mmap = unsafe { boot::exit_boot_services(None) };
            let mut pma = PhysicalMemoryAllocator::new(&mmap);
            let table = paging::init(&mmap, &mut pma);

            Ok(())
        },
        None => Err(BootError::AcpiNotFound),
    }
}


