///! The physical memory allocator handles allocation of physical frames, it does not concern
///! itself with virtual memory.

use crate::helpers::*;

use uefi::mem::memory_map::{MemoryType, MemoryMap, MemoryMapOwned};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;
use spin::Mutex;

static PMA: Mutex<PhysicalMemoryAllocator> = Mutex::new(PhysicalMemoryAllocator::new());

#[inline(always)]
pub fn init(mmap: &MemoryMapOwned) {
    PMA.lock().init(mmap)
}

#[inline(always)]
pub fn alloc(frames: usize) -> *const () {
    PMA.lock().alloc(frames)
}

#[inline(always)]
pub unsafe fn free(address: *const (), frames: usize) {
    unsafe {
        PMA.lock().free(address, frames)
    }
}

pub struct PhysicalMemoryAllocator {
    bitmap: [u128; 2048],
}

impl PhysicalMemoryAllocator {
    /// Create a new physical memory allocator
    pub const fn new() -> PhysicalMemoryAllocator {
        PhysicalMemoryAllocator {
            bitmap: [0u128; 2048],
        }
    }

    /// Initialize the physical memory allocator
    pub fn init(&mut self, mmap: &MemoryMapOwned) {
        log!("initializing pma with {} mmap entries", mmap.len());

        for descriptor in mmap.entries() {
            if descriptor.ty == MemoryType::CONVENTIONAL {
                let base = descriptor.phys_start as usize / 4096;

                for frame in 0..descriptor.page_count {
                    let bit = base + frame as usize;

                    self.bitmap[bit / u128::BITS as usize] &= !(1u128 << (bit % u128::BITS as usize));
                }
            }
        }
    }

    // TODO: implement allocation across multiple chunks
    /// Allocate a continuous set of physical frames within a 128 frame chunk
    pub fn alloc(&mut self, frames: usize) -> *const () {
        assert_ne!(frames, 0);

        for (index, chunk) in self.bitmap.iter_mut().enumerate() {
            if *chunk != u128::MAX {
                let bit = chunk.trailing_ones();
                let mask = ((1u128 << frames) - 1) << bit;

                if *chunk & mask == 0u128 {
                    *chunk |= mask;

                    let address = ((index * 128) + bit as usize) * 4096;

                    return address as *const ();
                }
            }
        }

        panic!("out of memory")
    }

    /// Free a continuous set of physical frames.
    ///
    /// free is unsafe because the caller must ensure that the address and frame count is valid.
    pub unsafe fn free(&mut self, address: *const (), frames: usize) {
        assert_ne!(frames, 0);

        let base = address as usize / 4096;

        for bit in base..base + frames {
            self.bitmap[bit / u128::BITS as usize] &= !(1u128 << (bit % u128::BITS as usize));
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalMemoryAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        log!("try allocating frame");

        let frame = self.alloc(1)?;

        log!("allocated frame: {:x?}", frame);

        Some(PhysFrame::containing_address(PhysAddr::new(frame as u64)))
    }
}


