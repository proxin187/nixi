//! Utilites for working with control registers

use core::arch::asm;

/// Set the Cr3 register for 4-level paging, the Cr3 register points to a PML4 table, the pointer must be a physical address
pub fn cr3_set(pml4: u64) {
    assert!(pml4 % 4096 == 0);

    unsafe {
        asm!(
            "mov cr3, {pml4}",
            pml4 = in(reg) pml4,
        );
    }
}

/// Read the Cr3 register
pub fn read_cr3() -> u64 {
    let mut pml4 = 0;

    unsafe {
        asm!(
            "mov {pml4}, cr3",
            pml4 = out(reg) pml4,
        );
    }

    pml4
}

/// Cr4 feature flags
pub struct Cr4Flags;

impl Cr4Flags {
    pub const FSGSBASE: u64 = 1 << 16;
    pub const OSXSAVE: u64 = 1 << 18;
}

/// Mask a processor feature in Cr4
#[inline]
pub fn cr4_mask(flag: u64) {
    unsafe {
        asm!(
            "mov {reg}, cr4",
            "or {reg}, {flag:r}",
            "mov cr4, {reg}",
            reg = out(reg) _,
            flag = in(reg) flag,
        );
    }
}

/// XCr0 flags
pub struct XCr0Flags;

impl XCr0Flags {
    pub const X87_FPU: u64 = 1;
    pub const SSE_STATE: u64 = 1 << 1;
}

/// Set the value of XCr0
pub fn xcr0_set(flag: u64) {
    let low_flag = flag as u32;
    let high_flag = (flag >> 32) as u32;

    unsafe {
        asm!(
            "xsetbv",
            in("ecx") 0,
            in("eax") low_flag,
            in("edx") high_flag,
        );
    }
}
