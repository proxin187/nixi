//! Startup and boot process

use crate::helpers::*;

use crate::arch::x86_64::{self, tables};
use crate::irq;
use crate::loader::Loader;
use crate::loader::error::LoaderError;
use crate::mem::pma;
use crate::scheduler::context;
use crate::vfs;
use crate::vfs::MountSource;

use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::cfg::ConfigTableEntry;

use core::cell::OnceCell;

use spin::Mutex;

/// Kernel region in memory
static KERNEL_REGION: Mutex<OnceCell<KernelRegion>> = Mutex::new(OnceCell::new());

/// Get the kernel region
pub fn kernel_region() -> KernelRegion {
    *KERNEL_REGION
        .lock()
        .get()
        .expect("kernel region not initialized")
}

/// Base address and size in bytes of kernel region in memory
#[derive(Debug, Clone, Copy)]
pub struct KernelRegion {
    pub base: u64,
    pub bytes: u64,
}

impl KernelRegion {
    pub fn new() -> KernelRegion {
        let handle = boot::image_handle();
        let image = boot::open_protocol_exclusive::<LoadedImage>(handle)
            .expect("failed to open loaded image protocol");

        let (base, bytes) = image.info();

        KernelRegion {
            base: base as u64,
            bytes,
        }
    }
}

unsafe impl Sync for KernelRegion {}
unsafe impl Send for KernelRegion {}

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

/// Boot and initialize the kernel
pub fn boot() -> ! {
    KERNEL_REGION
        .lock()
        .try_insert(KernelRegion::new())
        .expect("failed to initialize kernel region");

    log!("kernel region: {:x?}", KERNEL_REGION.lock().get());

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
