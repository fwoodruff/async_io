use std::{
    collections::VecDeque,
    sync::Mutex
};
use super::{
    SharedTask, 
    State, 
    super::task::*
};

pub struct PendingTasks {
    pub(super) to_poll : Mutex<VecDeque<SharedTask>>,
}

impl PendingTasks {
    pub fn new() -> Self {
        Self {
            to_poll : Mutex::new(VecDeque::new()),
        }
    }

    pub fn empty(&self) -> bool {
        let guard = self.to_poll.lock().unwrap();
        guard.is_empty()
    }

    pub fn pop(&self) -> Option<SharedTask> {
        let mut guard = self.to_poll.lock().unwrap();
        guard.pop_front()
    }
    
    pub fn push(&self, task : SharedTask) {
        let mut guard = self.to_poll.lock().unwrap();
        guard.push_back(task);
    }
}

pub fn current_state() -> *mut State {
    let current_task = CURRENT.with(|x| { x.borrow().as_ref().unwrap().upgrade().unwrap()});
    let v = current_task.b.lock().unwrap();
    let res = v.producer as *mut State;
    res
}