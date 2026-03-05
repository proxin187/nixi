///! The physical memory allocator handles allocation of physical frames, it does not concern
///! itself with virtual memory.

use uefi::mem::memory_map::{MemoryType, MemoryMap, MemoryMapOwned};

use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;


pub struct PhysicalMemoryAllocator {
    bitmap: [u128; 2048],
}

impl PhysicalMemoryAllocator {
    /// Initialize the physical memory allocator
    pub fn new(mmap: &MemoryMapOwned) -> PhysicalMemoryAllocator {
        let mut bitmap = [0u128; 2048];

        for descriptor in mmap.entries() {
            crate::kernel::drivers::serial::write_fmt(format_args!("descriptor: {:x?}\n", descriptor));

            if descriptor.ty != MemoryType::CONVENTIONAL {
                let base = descriptor.phys_start as usize / 4096;

                for frame in 0..descriptor.page_count {
                    let bit = base + frame as usize;

                    // TODO: the bug is that we panic here, the issue was never the loop. lets
                    // implement a panic handler that writes to the serial
                    bitmap[bit / u128::BITS as usize] |= 1u128 << (bit % u128::BITS as usize);
                }
            }
        }

        crate::kernel::drivers::serial::write_fmt(format_args!("done\n"));

        PhysicalMemoryAllocator {
            bitmap,
        }
    }

    // TODO: implement allocation across multiple chunks
    /// Allocate a continuous set of physical frames within a 128 frame chunk
    pub fn alloc(&mut self, frames: usize) -> Option<*const ()> {
        assert_ne!(frames, 0);

        for (index, chunk) in self.bitmap.iter_mut().enumerate() {
            if *chunk != u128::MAX {
                let bit = chunk.trailing_ones();
                let mask = ((1u128 << frames) - 1) << bit;

                if *chunk & mask == 0u128 {
                    *chunk |= mask;

                    let address = ((index * 128) + bit as usize) * 4096;

                    return Some(address as *const ());
                }
            }
        }

        None
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
        let frame = self.alloc(1)?;

        Some(PhysFrame::containing_address(PhysAddr::new(frame as u64)))
    }
}


