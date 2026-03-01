///! The physical memory allocator handles allocation of page frames

use uefi::mem::memory_map::{MemoryType, MemoryMap, MemoryMapOwned};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame};


pub struct PhysicalMemoryAllocator {
    frames: [u128; 2048],
}

impl PhysicalMemoryAllocator {
    pub fn new(mmap: MemoryMapOwned) -> PhysicalMemoryAllocator {
        let mut frames = [0u128; 2048];

        // TODO: instead of both filling the frames and identity mapping in a single iteration, we
        // can instead have two separate stages, a first stage where it fills the frames so that
        // its possible to use the allocator, then we do the identity mapping
        //
        // the other problem then will be that how do we identity map the first allocation?
        // specifically, do we need to identity map the page tables?

        for descriptor in mmap.entries() {
            match descriptor.ty {
                MemoryType::CONVENTIONAL => {
                    let base = descriptor.phys_start as usize / 4096;

                    for frame in 0..descriptor.page_count {
                        let bit = base + frame as usize;

                        frames[bit / u128::BITS as usize] |= 1u128 << (bit % u128::BITS as usize);
                    }
                },
                _ => {
                    // TODO: here we have to identity map the memory
                },
            }
        }

        PhysicalMemoryAllocator {
            frames,
        }
    }

    pub fn alloc(&mut self, pages: usize) {
    }

    pub fn free(&mut self, page: Page) {
        assert_eq!(page.size(), 4096);
    }
}


