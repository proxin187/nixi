

#[derive(Debug)]
pub enum BootError {
    AcpiNotFound,
    UefiError(uefi::Error<()>),
}

impl core::fmt::Display for BootError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            BootError::AcpiNotFound => f.write_str("unable to find ACPI in config table"),
            BootError::UefiError(err) => f.write_fmt(format_args!("uefi: {}", err)),
        }
    }
}

impl core::error::Error for BootError {}

impl From<uefi::Error<()>> for BootError {
    fn from(err: uefi::Error<()>) -> BootError {
        BootError::UefiError(err)
    }
}


