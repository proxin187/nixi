use crate::kernel::mem::error::MemoryError;


#[derive(Debug)]
pub enum BootError {
    AcpiNotFound,
    UefiError(uefi::Error<()>),
    MemoryError(MemoryError),
}

impl core::fmt::Display for BootError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            BootError::AcpiNotFound => f.write_str("unable to find ACPI in config table"),
            BootError::UefiError(err) => f.write_fmt(format_args!("uefi: {}", err)),
            BootError::MemoryError(err) => f.write_fmt(format_args!("memory: {}", err)),
        }
    }
}

impl core::error::Error for BootError {}

impl From<uefi::Error<()>> for BootError {
    fn from(err: uefi::Error<()>) -> BootError {
        BootError::UefiError(err)
    }
}

impl From<MemoryError> for BootError {
    fn from(err: MemoryError) -> BootError {
        BootError::MemoryError(err)
    }
}


