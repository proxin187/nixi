//! The scheduler implements multitasking on a single cpu

pub mod context;
pub mod proc;
pub mod task;

use crate::mem::paging::PageTableEntry;

use context::Context;

use proc::{ProcId, ProcManager};
use task::{Task, TaskId, TaskManager};

use spin::{Lazy, Mutex};

use crate::mem::paging::PageTable;

/// The global scheduler
static SCHEDULER: Lazy<Mutex<Scheduler>> = Lazy::new(|| Mutex::new(Scheduler::new()));

/// Call a closure with scheduler
pub fn with_scheduler<F: FnOnce(&mut Scheduler) -> R, R>(f: F) -> R {
    let mut scheduler = SCHEDULER.lock();

    f(&mut scheduler)
}

/// The scheduler manages tasks and processes
pub struct Scheduler {
    task_manager: TaskManager,
    proc_manager: ProcManager,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Scheduler {
        Scheduler {
            task_manager: TaskManager::new(),
            proc_manager: ProcManager::new(),
        }
    }

    // TODO: nothing happens after we load the page table because the kernel code isnt mapped, so when it then tries to execute the kernel code, it just faults

    /// Return the initial task and page table
    pub fn get_inital_task(&self) -> (&Task, *const PageTableEntry) {
        let task = self
            .task_manager
            .get_task(TaskId::new(1))
            .expect("no initial task");

        let page_table = self
            .proc_manager
            .get_pt_ptr(task.proc_id)
            .expect("page table not found for task");

        (task, page_table)
    }

    /// Return the next context and page table
    pub fn switch(&mut self, ctx: Context) -> (Context, *const PageTableEntry) {
        let (proc_id, ctx) = self.task_manager.switch(ctx);

        let page_table = self
            .proc_manager
            .get_pt_ptr(proc_id)
            .expect("page table not found for process");

        (ctx, page_table)
    }

    /// Get kernel and user stack for a task
    pub fn get_task(&self, task_id: TaskId) -> &Task {
        self.task_manager.get_task(task_id).expect("task not found")
    }

    /// Get the page table of a process
    pub fn get_pt(&mut self, proc_id: ProcId) -> Option<&mut PageTable> {
        self.proc_manager.get_pt(proc_id)
    }

    /// Create a new process and return its process id
    pub fn create_proc(&mut self) -> ProcId {
        self.proc_manager.create()
    }

    /// Create a new task for a process and return its task id
    pub fn create_task(&mut self, proc_id: ProcId, entry: u64, privilege_level: u8) -> TaskId {
        let task_id = self
            .task_manager
            .create(Task::new(proc_id, entry, privilege_level));

        self.proc_manager.adopt_task(proc_id, task_id);

        task_id
    }
}
