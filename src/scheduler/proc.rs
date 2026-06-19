//! A process is a set of tasks with a shared memory space

use super::task::TaskId;

use crate::mem::paging::PageTable;
use crate::vfs::OwnedPath;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// A process id points to a process
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcId(u128);

impl ProcId {
    /// Create a new process id
    pub fn new(id: u128) -> ProcId {
        ProcId(id)
    }
}

/// A process, a single process can have multiple tasks
pub struct Proc {
    tasks: Vec<TaskId>,
    page_table: PageTable,
    cwd: OwnedPath,
}

impl Proc {
    /// Create a new process
    pub fn new() -> Proc {
        let page_table = PageTable::new();

        Proc {
            tasks: Vec::new(),
            page_table,
            cwd: OwnedPath::from("/"),
        }
    }

    /// Get the current working directory
    pub fn cwd(&self) -> &OwnedPath {
        &self.cwd
    }

    /// Adopt a task
    pub fn adopt_task(&mut self, task_id: TaskId) {
        self.tasks.push(task_id);
    }
}

/// The process manager stores processes for efficient lookup
pub struct ProcManager {
    procs: BTreeMap<ProcId, Proc>,
    next_id: u128,
}

impl ProcManager {
    /// Create a new process manager
    pub fn new() -> ProcManager {
        ProcManager {
            procs: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// Create a new process and return its process id
    pub fn create(&mut self) -> ProcId {
        self.next_id += 1;

        let proc_id = ProcId::new(self.next_id);

        self.procs.insert(proc_id, Proc::new());

        proc_id
    }

    /// Load the page table of a specific process
    pub fn load_pt(&self, proc_id: ProcId) {
        if let Some(proc) = self.procs.get(&proc_id) {
            proc.page_table.load();
        }
    }

    /// Get a mutable reference to the page table of a process
    pub fn get_pt(&mut self, proc_id: ProcId) -> Option<&mut PageTable> {
        self.procs
            .get_mut(&proc_id)
            .map(|proc| &mut proc.page_table)
    }

    /// Adopt a task into a process. This does not modify the task and assumes the task has its proc_id field set correctly
    pub fn adopt_task(&mut self, proc_id: ProcId, task_id: TaskId) {
        if let Some(proc) = self.procs.get_mut(&proc_id) {
            proc.adopt_task(task_id);
        }
    }
}
