use crate::kernel::mem::pma;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;


#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

/// The block header is at the start of each free block, a block can have any size and alignment
#[repr(C, packed)]
struct BlockHeader {
    next: Option<*mut BlockHeader>,
    size: usize,
}

impl BlockHeader {
    /// Find the first block with atleast size bytes
    pub fn find(&mut self, size: usize) -> Option<*mut BlockHeader> {
        if self.size >= size {
            Some(ptr::from_mut(self))
        } else {
            self.next.and_then(|next| unsafe { (*next).find(size) })
        }
    }
}

pub struct Allocator {
    first: Option<*mut BlockHeader>,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            first: None,
        }
    }

    pub fn prepare(&self, size: usize) -> *mut BlockHeader {
        match self.first.and_then(|first| unsafe { (*first).find(size) }) {
            Some(header) => header,
            None => {
                let header = pma::alloc(size / 4096);
                // TODO: write header into header

                header as *mut BlockHeader
            },
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(header) = self.first {
            unsafe {
                (*header).find(layout.size());
            }
        }

        // implementation steps:
        // if there arent enough frames then we must allocate enough
        //
        // find a continuous region of memory inside those pages that is big enough, we can
        // probably do this by finding the lowest available address for the start and just add the
        // size to it for the end.
        //
        // implementation notes:
        // only frames which have been allocated by the pma, but are not allocated by the allocator
        // should have a block header
        //
        // blocks dont have to be frames, infact in most cases they wont be, eg. in cases where we
        // need 5 frames we would have it as a single block, this significantly simplifies the
        // implementation

        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // implementation steps:
        // detect complete frames and free them to the pma
        //
        // incomplete frames should be added to the frame linked list
        //
        // implementation notes:
        // we should also do e

        todo!()
    }
}

unsafe impl Sync for Allocator {}


