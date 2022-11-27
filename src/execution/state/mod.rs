pub mod poll;
pub mod execute;

use super:: {
    state::{
        execute::PendingTasks, 
        poll::SharedPoller
    },
    SharedTask,
    Task,
};
use std::sync::Arc;

pub(in crate::execution) struct State {
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

    pub(super) fn spawn(&self, task: SharedTask) {
        self.push(task);
        self.pl.notify();
        self.cv.notify_one();
    }

    pub(super) fn is_empty(&self) -> bool {
        self.cx.empty() && self.pl.is_empty()
    }

    pub(super) fn wait(&self) {
        let mut cx_guarded = self.cx.to_poll.lock().unwrap();
        while cx_guarded.is_empty() && !( self.pl.is_empty() && cx_guarded.is_empty()) {
            cx_guarded = self.cv.wait(cx_guarded).unwrap();
        }
        drop(cx_guarded);
        self.cv.notify_one();
    }

    pub(super) fn push(&self, task: SharedTask) {
        self.cx.push(task);
    }
    
    pub(super) fn pop(&self) -> Option<SharedTask> {
        self.cx.pop()
    }

    pub(super) fn poll(&self) -> Option<Vec<Arc<Task>>> {
        self.pl.poll()
    }

    pub(super) fn notify(&self) {
        self.pl.notify();
        self.cv.notify_one();
    }
    
}