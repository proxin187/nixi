//! Memory errors

use thiserror::Error;

/// A generic error type for all memory submodules
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("attempted to descend a mapped page (nested mapping is not possible)")]
    DescendedMappedPage,
}
