//! Code for working with 64-bit paging

use core::ops::Range;

use super::error::MemoryError;
use super::pma;

use crate::arch::x86_64;
use crate::arch::x86_64::registers;

/// Represents the size of a page, modern processors support up to 1GiB pages
#[derive(Debug, Clone, Copy)]
pub enum PageSize {
    Page4KiB,
    Page2MiB,
    Page1GiB,
}

impl PageSize {
    /// Get the alignment of a particular page size
    pub fn align(&self) -> u64 {
        match self {
            PageSize::Page4KiB => 0x1000,
            PageSize::Page2MiB => 0x200000,
            PageSize::Page1GiB => 0x40000000,
        }
    }

    /// Get the total number of levels excluding the first
    pub fn levels(&self) -> u8 {
        match self {
            PageSize::Page4KiB => 3,
            PageSize::Page2MiB => 2,
            PageSize::Page1GiB => 1,
        }
    }
}

/// Represents the entire recursive page table.
///
/// The page tables are in a 4-level radix tree structure
pub struct PageTable {
    pub pml4: *mut PageTableEntry,
}

impl PageTable {
    /// Create a new empty page table
    pub fn new() -> PageTable {
        PageTable {
            pml4: pma::alloc_zeroed(1) as *mut PageTableEntry,
        }
    }

    /// Load the page table by setting the Cr3 register
    ///
    /// This a convencience wrapper for [cr3_set](registers::cr3_set)
    pub fn load(&self) {
        registers::cr3_set(self.pml4 as u64);
    }

    // TODO: do we really need the lookup?

    /// Lookup the physical address which a virtual address is mapped
    pub fn lookup(vaddr: u64) -> u64 {
        todo!()
    }

    /// Identity map some memory given a start address, a page count, flags and the page size
    pub fn identity_map(
        &mut self,
        addr: u64,
        pages: u64,
        flags: u64,
        size: PageSize,
    ) -> Result<(), MemoryError> {
        for page in 0..pages {
            let paddr = addr + (page * size.align());

            self.map(paddr, paddr, flags, size)?;
        }

        Ok(())
    }

    /// Map consecutive pages within a range
    pub fn map_consecutive_range(
        &mut self,
        vaddr: Range<u64>,
        paddr: u64,
        flags: u64,
        size: PageSize,
    ) -> Result<(), MemoryError> {
        for page in 0..(vaddr.end - vaddr.start) / size.align() {
            let offset = page * size.align();

            self.map(vaddr.start + offset, paddr + offset, flags, size)?;
        }

        Ok(())
    }

    /// Map a virtual address to a physical address, both addresses must be aligned to the page size
    ///
    /// This is purely a safe convenience wrapper for [create_map](PageTable::create_map)
    pub fn map(
        &mut self,
        vaddr: u64,
        paddr: u64,
        flags: u64,
        size: PageSize,
    ) -> Result<(), MemoryError> {
        if let Some(misaligned) = (vaddr % size.align() != 0)
            .then_some(vaddr)
            .or_else(|| (paddr % size.align() != 0).then_some(paddr))
        {
            Err(MemoryError::MisalignedMap(misaligned))
        } else {
            unsafe { self.create_map(vaddr, paddr, flags, size, size.levels(), self.pml4) }
        }
    }

    /// Recursively create a page table mapping for a virtual address in the radix tree
    ///
    /// The level must correspond to the amount of levels remainding after the table (eg. PML4 would be level 3 in a 4KiB page map).
    unsafe fn create_map(
        &mut self,
        vaddr: u64,
        paddr: u64,
        flags: u64,
        size: PageSize,
        depth: u8,
        table: *mut PageTableEntry,
    ) -> Result<(), MemoryError> {
        let level = size.levels() as u64 - depth as u64;
        let index = (vaddr >> (12 + ((3 - level) * 9))) & 0x1ff;

        unsafe {
            let entry = table.byte_add(index as usize * core::mem::size_of::<PageTableEntry>());

            if depth > 0 {
                if !(*entry).is_present() {
                    *entry = PageTableEntry::new(
                        pma::alloc_zeroed(1) as u64,
                        flags | PageTableEntryFlags::PRESENT,
                    );
                } else if (*entry).is_page_map() {
                    return Err(MemoryError::DescendedMappedPage);
                }

                self.create_map(
                    vaddr,
                    paddr,
                    flags,
                    size,
                    depth - 1,
                    (*entry).physical_address() as *mut PageTableEntry,
                )?;
            } else {
                *entry = PageTableEntry::new(
                    paddr,
                    flags | PageTableEntryFlags::PRESENT | PageTableEntryFlags::PAGE_SIZE,
                )
            }
        }

        Ok(())
    }
}

unsafe impl Send for PageTable {}
unsafe impl Sync for PageTable {}

/// Page table entry flags, only flags which are common between PML4E, PDPTE, PDE and PTE are represented
pub struct PageTableEntryFlags;

impl PageTableEntryFlags {
    /// Must be 1 on all pages which you wish to use
    const PRESENT: u64 = 1;

    /// Indicates the ability to write to memory inside this page
    pub const WRITE: u64 = 1 << 1;

    /// Indicates the ability for usermode (ring 3) access to this page
    pub const USER: u64 = 1 << 2;

    /// If 1 then this will map the entry as a page at the corresponding level, if 0 then the entry will reference a page table at a lower level
    const PAGE_SIZE: u64 = 1 << 7;
}

/// A generic page table entry, this can either be the PML4E, PDPTE, PDE or the PTE
#[repr(C, packed)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Create a new page table entry with a physical address and flags
    pub fn new(paddr: u64, flags: u64) -> PageTableEntry {
        PageTableEntry {
            entry: (flags & 0xff) | (paddr & !0xfff),
        }
    }

    /// Returns the physical address field of the entry
    ///
    /// This will panic if the PAGE_SIZE flag is set to prevent accidental misuse
    pub fn physical_address(&self) -> u64 {
        assert!(!self.is_page_map());

        self.entry & (((1u64 << x86_64::physical_address_width()) - 1) << 12)
    }

    /// Returns true if the PAGE_SIZE flag is set
    pub fn is_page_map(&self) -> bool {
        self.entry & PageTableEntryFlags::PAGE_SIZE != 0
    }

    /// Returns true if the present flag is set
    pub fn is_present(&self) -> bool {
        self.entry & 1 == 1
    }
}
