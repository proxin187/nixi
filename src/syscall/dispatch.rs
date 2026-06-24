//! The syscall dispatch

use crate::syscall::error::SyscallError;
use crate::vfs::fd::FileDescriptorId;
use crate::vfs::{self, OwnedPath};

use core::slice;

/// Each syscall has a unique syscall number
pub struct SyscallNumber;

impl SyscallNumber {
    /// The open syscall opens a new file descriptor
    pub const OPEN: u64 = 0;

    /// The close syscall closes a file descriptor
    pub const CLOSE: u64 = 1;

    /// The read syscall reads a file descriptor
    pub const READ: u64 = 2;

    /// The write syscall writes a file descriptor
    pub const WRITE: u64 = 3;
}

/// Dispatch a syscall to a handler
pub fn dispatch(syscall: u64, args: [u64; 4]) -> Result<u64, SyscallError> {
    crate::log!("syscall={}, args={:?}", syscall, args);

    match syscall {
        SyscallNumber::OPEN => {
            let path = unsafe { OwnedPath::from_raw_parts(args[0] as *mut u8, args[1] as usize) };
            let fd_id = vfs::with_vfs(|vfs| vfs.open(path)).map(|fd| fd.value())?;

            Ok(fd_id)
        }
        SyscallNumber::CLOSE => {
            vfs::with_vfs(|vfs| vfs.close(FileDescriptorId::new(args[0])));

            Ok(0)
        }
        SyscallNumber::READ => {
            let buf = unsafe { slice::from_raw_parts_mut(args[1] as *mut u8, args[2] as usize) };
            let read = vfs::with_vfs(|vfs| vfs.read(FileDescriptorId::new(args[0]), buf))?;

            Ok(read)
        }
        SyscallNumber::WRITE => {
            let buf = unsafe { slice::from_raw_parts(args[1] as *const u8, args[2] as usize) };
            let write = vfs::with_vfs(|vfs| vfs.write(FileDescriptorId::new(args[0]), buf))?;

            Ok(write)
        }
        _ => Err(SyscallError::NotFound),
    }
}
