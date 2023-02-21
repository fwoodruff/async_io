use std::{
    collections::VecDeque,
    sync::Mutex
};
use crate::implementation::SharedTask;
use crate::implementation::CURRENT;

use super::blockingstate::State;

// the queue of tasks that can resume
#[derive(Default)]
pub(crate) struct PendingTasks {
    pub(super) to_poll : Mutex<VecDeque<SharedTask>>,
}

impl PendingTasks {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    // try take a task
    pub(super)
    fn pop(&self) -> Option<SharedTask> {
        let mut guard = self.to_poll.lock().unwrap();
        guard.pop_front()
    }
    
    // try add a task
    pub(super) fn push(&self, task : SharedTask) {
        let mut guard = self.to_poll.lock().unwrap();
        guard.push_back(task);
    }
    
}

// for the current task running, retrieve the executor it is running on
pub(crate)
unsafe fn current_state() -> &'static mut State {
    let current_task = CURRENT.with(|x| { x.borrow().as_ref().unwrap().upgrade().unwrap()});
    let v = current_task.b.lock().unwrap();
    let res = v.producer as *mut State;
    &mut (*res)
}
