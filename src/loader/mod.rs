//! The program loader

pub mod error;

use core::alloc::Layout;
use error::LoaderError;

use crate::mem::paging::{PageSize, PageTableEntryFlags};
use crate::parse::elf::{ElfObject, ProgramHeader};
use crate::scheduler;
use crate::scheduler::proc::ProcId;
use crate::vfs::error::VfsError;
use crate::vfs::{self, OwnedPath};

use alloc::vec;
use alloc::vec::Vec;

/// Translate p_flags into page table entry flags
#[inline]
fn translate_pflags_to_pte(p_flags: u32) -> u64 {
    let mut flags = 0;

    if p_flags & ProgramHeader::PF_W != 0 {
        flags |= PageTableEntryFlags::WRITE;
    }

    flags
}

/// The program loader loads a program from ELF into memory
pub struct Loader {
    bytes: Vec<u8>,
    proc_id: ProcId,
}

impl Loader {
    /// Prepare a loader for a program from the file system
    pub fn from_fs(path: &str) -> Result<Loader, LoaderError> {
        let bytes = vfs::with_vfs(|vfs| -> Result<Vec<u8>, VfsError> {
            let fd = vfs.open(OwnedPath::from(path))?;

            let metadata = vfs.metadata(fd)?;

            let mut buf = vec![0u8; metadata.length as usize];

            vfs.read(fd, &mut buf)?;

            Ok(buf)
        })?;

        let proc_id = scheduler::with_scheduler(|scheduler| scheduler.create_proc());

        Ok(Loader { bytes, proc_id })
    }

    /// Load a program header into memory
    fn load_header(&self, header: &ProgramHeader) -> Result<(), LoaderError> {
        let size = header.p_memsz.next_multiple_of(4096) as usize;
        let layout = Layout::from_size_align(size, header.p_align as usize)
            .map_err(|_| LoaderError::InvalidLayout)?;

        let buf = unsafe { core::slice::from_raw_parts_mut(alloc::alloc::alloc(layout), size) };

        buf[..header.p_filesz as usize].copy_from_slice(
            &self.bytes
                [header.p_offset as usize..header.p_offset as usize + header.p_filesz as usize],
        );

        scheduler::with_scheduler(|scheduler| {
            let page_table = scheduler
                .get_pt(self.proc_id)
                .expect("failed to get page table for new process");

            // TODO: we should not identity map anything in the process, currently all processes are identity mapped from the start, this is wrong, we will have to remove this.
            //
            // The only mapping should happen here, or when the process allocates memory

            crate::log!("header: {:?}", header);

            page_table.map_consecutive_range(
                header.p_vaddr..header.p_vaddr + buf.len() as u64,
                buf.as_ptr() as u64,
                translate_pflags_to_pte(header.p_flags),
                PageSize::Page4KiB,
            );
        });

        Ok(())
    }

    /// Load the program into memory and create task
    pub fn load(&self) -> Result<(), LoaderError> {
        let object = ElfObject::parse(&self.bytes).ok_or(LoaderError::InvalidElf)?;
        let headers = object.program_headers();

        for header in headers.filter(|header| header.p_type == ProgramHeader::PT_LOAD) {
            self.load_header(header)?;
        }

        scheduler::with_scheduler(|scheduler| {
            scheduler.create_task(self.proc_id, object.entry(), 3);
        });

        Ok(())
    }
}
