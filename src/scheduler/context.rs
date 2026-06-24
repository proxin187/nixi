//! Context switching

use super::SCHEDULER;

use crate::arch::x86_64::interrupt::StackFrame;
use crate::arch::x86_64::tables;
use crate::helpers::*;
use crate::mem::paging;
use crate::scheduler;

use core::arch::asm;
use core::sync::atomic::Ordering;

/// Segment registers
#[repr(C, packed)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Segments {
    pub fs: u64,
    pub gs: u64,
}

/// General purpose registers
#[repr(C, packed)]
#[derive(Default, Debug, Clone, Copy)]
pub struct GeneralPurpose {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
}

/// Saved context of a task
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Context {
    pub segments: Segments,
    pub general: GeneralPurpose,
    pub stack_frame: StackFrame,
}

/// Enter usermode, the scheduler will panic if it has no tasks when this is called
pub fn enter_usermode() -> ! {
    log!("enter usermode");

    let (page_table, stack_frame, kernel_stack) = scheduler::with_scheduler(|scheduler| {
        let (task, page_table) = scheduler.get_inital_task();

        (page_table, task.ctx.stack_frame, unsafe {
            task.kernel_stack.0.as_ptr().add(task.kernel_stack.0.len())
        })
    });

    tables::set_kernel_stack(kernel_stack);

    unsafe {
        asm!(
            "mov cr3, {page_table}",
            "push {ss}",
            "push {rsp}",
            "push {rflags}",
            "push {cs}",
            "push {rip}",
            "iretq",
            page_table = in(reg) page_table,
            ss = in(reg) stack_frame.ss,
            rsp = in(reg) stack_frame.rsp,
            rflags = in(reg) stack_frame.rflags.flags,
            cs = in(reg) stack_frame.cs,
            rip = in(reg) stack_frame.rip,
            options(noreturn),
        );
    }
}

/// Switch is called directly from assembly in the interrupt handler
#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn switch(ctx: *mut Context) {
    let mut scheduler = SCHEDULER.lock();

    log!("entry: {:x?}", unsafe { *ctx });

    unsafe {
        let (context, page_table) = scheduler.switch(*ctx);

        paging::USER_PAGE_TABLE.store(page_table as u64, Ordering::Relaxed);

        *ctx = context;
    }

    log!("exit: {:x?}", unsafe { *ctx });
}
