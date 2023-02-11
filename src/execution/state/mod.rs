mod poll;
pub mod execute;

use super:: {
    state::{
        execute::PendingTasks, 
        poll::SharedPoller
    },
    SharedTask,
    Task,
};

// The internal state for the Executor. It includes tasks that can be run, and tasks that are waiting for network IO
pub
struct State {
    cx : PendingTasks,
    cv : std::sync::Condvar,
    pub pl : SharedPoller,
}

impl State {
    pub fn new() -> Self {
        Self {
            cx : PendingTasks::new(),
            cv : std::sync::Condvar::new(),
            pl : SharedPoller::new(),
        }
    }

    // adds new work and unblocks the thread pool
    pub(super)
    fn spawn(&self, task: SharedTask) {
        self.push(task);
        self.pl.notify();
        self.cv.notify_one();
    }

    // if false, that means that there are no tasks dormant or otherwise associated with this executor
    // meaning the executor can be dropped
    pub(super)
    fn is_empty(&self) -> bool {
        self.cx.empty() && self.pl.is_empty()
    }

    // tells thread pool to wait for a task
    pub(super)
    fn wait(&self) {
        let mut cx_guarded = self.cx.to_poll.lock().unwrap();
        while cx_guarded.is_empty() && !( self.pl.is_empty() && cx_guarded.is_empty()) {
            cx_guarded = self.cv.wait(cx_guarded).unwrap();
        }
        drop(cx_guarded);
        self.cv.notify_one();
    }

    // Add a new task to the thread pool
    pub(super)
    fn push(&self, task: SharedTask) {
        self.cx.push(task);
    }
    
    // pop a task from the thread pool for execution
    pub(super) fn pop(&self) -> Option<SharedTask> {
        self.cx.pop()
    }

    // poll the polling context for tasks that can resume
    pub(super) fn poll(&self) -> Option<Vec<SharedTask>> {
        self.pl.poll()
    }

    // unblock the polling context and the thread pool
    pub(super)
    fn notify(&self) {
        self.pl.notify();
        self.cv.notify_one();
    }
    
}