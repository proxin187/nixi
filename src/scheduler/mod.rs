//! The scheduler implements multitasking on a single cpu

pub mod context;
pub mod proc;
pub mod task;

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

    /// Return the initial task and load its page table
    pub fn load_initial_task(&self) -> &Task {
        let task = self.task_manager.initial_task();

        self.proc_manager.load_pt(task.proc_id);

        task
    }

    /// Perform a context switch
    pub fn switch(&mut self, ctx: Context) -> Context {
        let (proc_id, ctx) = self.task_manager.switch(ctx);

        self.proc_manager.load_pt(proc_id);

        ctx
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
