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

use std::sync::atomic::{AtomicUsize, Ordering};


// The internal state for the Executor. It includes tasks that can be run, and tasks that are waiting for network IO
pub
struct State {
    cx : PendingTasks,
    cv : std::sync::Condvar,
    pub pl : SharedPoller,
    num_tasks : AtomicUsize,
}

impl State {
    pub fn new() -> Self {
        Self {
            cx : PendingTasks::new(),
            cv : std::sync::Condvar::new(),
            pl : SharedPoller::new(),
            num_tasks : AtomicUsize::new(1),
        }
    }

    pub(super)
    fn set_task(&self, task: SharedTask) {
        self.push(task);
        self.pl.notify();
        self.cv.notify_one();
    }

    // adds new work and unblocks the thread pool
    pub(super)
    fn spawn(&self, task: SharedTask) {
        self.num_tasks.fetch_add(1, Ordering::Release);
        self.set_task(task);
    }

    // At the end of a task's execution, its parent task must be alerted that it is done.
    // If a parent task is waiting for this task, we must resume the parent task.
    pub(super) 
    fn join_parent(&self, task : SharedTask) {
        self.num_tasks.fetch_sub(1, std::sync::atomic::Ordering::Acquire);
        let mut book = task.b.lock().unwrap();
        
        let mut parent_waiting = false;
        let task_parent = book.parent.take();
        {
            if let Some(ref parent_book_locked) = task_parent {
                let mut parent_book = parent_book_locked.b.lock().unwrap();
                let len = parent_book.children.len();
                parent_book.children.retain(|child| {
                    task.task_id() != *child
                });
                
                if len == parent_book.children.len() {
                    // this task was not removed from the parent, i.e. the parent is waiting
                    parent_waiting = true;
                }
            }
        }
        if parent_waiting {
            self.push(task_parent.unwrap());
        }
        if self.is_empty() {
            self.notify();
        }
    }


    // There are no tasks dormant or otherwise associated with this executor
    // meaning the executor itself may be dropped
    pub(super)
    fn is_empty(&self) -> bool {
        self.num_tasks.load(Ordering::Acquire) == 0
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