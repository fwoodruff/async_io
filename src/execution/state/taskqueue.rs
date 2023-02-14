use std::{
    collections::VecDeque,
    sync::Mutex
};
use crate::execution::SharedTask;
use crate::execution::CURRENT;

use super::blockingstate::State;

// the queue of tasks that can resume
#[derive(Default)]
pub struct PendingTasks {
    pub to_poll : Mutex<VecDeque<SharedTask>>,
}

impl PendingTasks {
    pub fn new() -> Self {
        Default::default()
    }

    // try take a task
    pub
    fn pop(&self) -> Option<SharedTask> {
        let mut guard = self.to_poll.lock().unwrap();
        guard.pop_front()
    }
    
    // try add a task
    pub fn push(&self, task : SharedTask) {
        let mut guard = self.to_poll.lock().unwrap();
        guard.push_back(task);
    }
    
}

// for the current task running, retrieve the executor it is running on
pub(in super::super)
unsafe fn current_state<'a>() -> &'a mut State {
    let current_task = CURRENT.with(|x| { x.borrow().as_ref().unwrap().upgrade().unwrap()});
    let v = current_task.b.lock().unwrap();
    let res = v.producer as *mut State;
    &mut (*res)
}