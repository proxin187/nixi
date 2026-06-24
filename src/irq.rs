//! Handling of interrupts

use crate::arch::x86_64::interrupt::{PageFaultErrorCode, StackFrame};
use crate::arch::x86_64::io;
use crate::drivers::pic8259;
use crate::mem::paging;
use crate::scheduler::context;

use crate::helpers::*;

use core::arch::naked_asm;

pub fn init() {
    pic8259::init(32);

    pic8259::enable_maskable_interrupts(&[0, 4]);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: StackFrame, error_code: u64) -> ! {
    error!(
        "double fault:\n{:#x?}\nerror code: {:#x?}",
        stack_frame, error_code
    );

    loop {}
}

pub extern "x86-interrupt" fn gp_fault(stack_frame: StackFrame, error_code: u64) {
    error!(
        "general protection fault:\n{:#x?}\nerror code: {:#x?}",
        stack_frame, error_code
    );

    loop {}
}

pub extern "x86-interrupt" fn page_fault(stack_frame: StackFrame, error_code: PageFaultErrorCode) {
    error!(
        "page fault:\n{:#x?}\nerror code: {}",
        stack_frame, error_code
    );

    loop {}
}

// TODO: we must also map both the user stack and kernel stack for the process, the kernel stack is technically kind of insecure to map, however, we dont care as long as it works

#[unsafe(naked)]
pub fn timer_interrupt() {
    naked_asm!(
        // save general purpose registers
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        // save FS/GS
        "rdgsbase rax",
        "push rax",
        "rdfsbase rax",
        "push rax",

        // load the kernel page table
        "mov rax, [{kernel_page_table}]",
        "mov cr3, rax",

        // call switch
        "mov rcx, rsp",
        "call {switch}",

        // call end_of_interrupt
        "mov rcx, 32",
        "call {end_of_interrupt}",

        // load the user page table
        "mov rax, [{user_page_table}]",
        "mov cr3, rax",

        // restore FS/GS
        "pop rax",
        "wrfsbase rax",
        "pop rax",
        "wrgsbase rax",

        // restore general purpose registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",

        "iretq",
        kernel_page_table = sym paging::KERNEL_PAGE_TABLE,
        user_page_table = sym paging::USER_PAGE_TABLE,
        switch = sym context::switch,
        end_of_interrupt = sym pic8259::end_of_interrupt,
    );
}

pub extern "x86-interrupt" fn com1_interrupt(_stack_frame: StackFrame) {
    unsafe {
        let byte = io::inb(0x3f8);

        crate::log!("pressed: {}", byte as char);

        // TODO: here we should write the byte to some sort of a device object which would be
        // mounted under something like /dev/kbd or something

        pic8259::end_of_interrupt(36);
    }
}
