//! A task is a function to be performed

use super::context::{Context, GeneralPurpose, Segments};
use super::proc::ProcId;

use crate::arch::x86_64;
use crate::arch::x86_64::interrupt::{RFlags, StackFrame};
use crate::arch::x86_64::tables;

use alloc::alloc::Layout;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// A task id points to a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u128);

impl TaskId {
    /// Create a new task id
    pub fn new(id: u128) -> TaskId {
        TaskId(id)
    }
}

/// A stack
#[repr(C, align(4096))]
pub struct Stack(pub [u8; 4096 * 4]);

/// A task will be run by the scheduler on interval
pub struct Task {
    pub proc_id: ProcId,
    pub ctx: Context,
    pub user_stack: Box<Stack>,
    pub kernel_stack: Box<Stack>,
    pub xsave: *mut u8,
}

impl Task {
    /// Create a new task with an entry point and privilege level
    pub fn new(proc_id: ProcId, entry: u64, privilege_level: u8) -> Task {
        let layout = Layout::from_size_align(x86_64::required_xsave_size() as usize, 64)
            .expect("xsave allocation shouldn't break alignment rules");

        let xsave = unsafe { alloc::alloc::alloc_zeroed(layout) };

        let user_stack = Box::new(Stack([0; 4096 * 4]));

        Task {
            proc_id,
            ctx: Context {
                segments: Segments::default(),
                general: GeneralPurpose::default(),
                stack_frame: StackFrame {
                    rip: entry,
                    cs: 0x18 | (privilege_level as u64 & 3),
                    rflags: RFlags::new(1 | (1 << 9)),
                    rsp: user_stack.0.as_ptr() as u64 + user_stack.0.len() as u64,
                    ss: 0x20 | (privilege_level as u64 & 3),
                },
            },
            user_stack,
            kernel_stack: Box::new(Stack([0; 4096 * 4])),
            xsave,
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            // NOTE: this uses a bogus layout since our allocator doesnt make use of the layout when deallocating
            alloc::alloc::dealloc(self.xsave, Layout::from_size_align_unchecked(8, 8));
        }
    }
}

unsafe impl Sync for Task {}
unsafe impl Send for Task {}

/// The task entry is an entry in the task manager
struct TaskEntry {
    task_id: TaskId,
    task: Task,
}

/// The task manager stores tasks and does allocation of task id's.
///
/// It uses a vector to store tasks, this means faster scheduling with the downside of slower lookup.
pub struct TaskManager {
    entries: Vec<TaskEntry>,
    next_task: usize,
    next_id: u128,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> TaskManager {
        TaskManager {
            entries: Vec::new(),
            next_task: 0,
            next_id: 0,
        }
    }

    /// Get a task from its task id
    pub fn get_task(&self, task_id: TaskId) -> Option<&Task> {
        self.entries
            .iter()
            .find(|entry| entry.task_id == task_id)
            .map(|entry| &entry.task)
    }

    /// Create a task
    pub fn create(&mut self, task: Task) -> TaskId {
        self.next_id += 1;

        let task_id = TaskId::new(self.next_id);

        self.entries.push(TaskEntry { task_id, task });

        task_id
    }

    /// Return the previous and next task index
    fn schedule(&mut self) -> (usize, usize) {
        let previous = self.next_task;

        self.next_task = (self.next_task + 1) % self.entries.len();

        (previous, self.next_task)
    }

    /// Save previous context and return new process id and context
    pub fn switch(&mut self, ctx: Context) -> (ProcId, Context) {
        let (previous, next) = self.schedule();

        self.entries[next].task.ctx = ctx;

        unsafe {
            core::arch::x86_64::_xsave(self.entries[previous].task.xsave, u64::MAX);
            core::arch::x86_64::_xrstor(self.entries[next].task.xsave, u64::MAX);

            let kernel_stack = &self.entries[next].task.kernel_stack;

            tables::set_kernel_stack(kernel_stack.0.as_ptr().add(kernel_stack.0.len()));
        }

        (self.entries[next].task.proc_id, self.entries[next].task.ctx)
    }
}
