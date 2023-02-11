use std::{
    collections::VecDeque,
    sync::Mutex
};
use super::{
    SharedTask, 
    State, 
    super::task::*
};

// the queue of tasks that can resume
#[derive(Default)]
pub struct PendingTasks {
    pub(super)
    to_poll : Mutex<VecDeque<SharedTask>>,
}

impl PendingTasks {
    pub(super) 
    fn new() -> Self {
        Default::default()
    }

    // there are no tasks
    pub(super)
    fn empty(&self) -> bool {
        let guard = self.to_poll.lock().unwrap();
        guard.is_empty()
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

// for the current task running, determine the executor it is running on
pub(in super::super)
fn current_state() -> *mut State {
    let current_task = CURRENT.with(|x| { x.borrow().as_ref().unwrap().upgrade().unwrap()});
    let v = current_task.b.lock().unwrap();
    let res = v.producer as *mut State;
    res
}