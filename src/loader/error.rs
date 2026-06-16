//! Loader errors

use crate::vfs::error::VfsError;

use thiserror::Error;

/// An error while loading a program
#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("invalid ELF binary")]
    InvalidElf,

    #[error("invalid layout for allocation")]
    InvalidLayout,

    #[error(transparent)]
    Vfs(#[from] VfsError),
}
