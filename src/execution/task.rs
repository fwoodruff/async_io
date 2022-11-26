
use std::pin::Pin;
use std::future::Future;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use super::*;

type ExecutorFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
pub type SharedTask = Arc<Task>;
type WeakTask = std::sync::Weak<Task>;

pub const NUMTHREADS : usize = 2;
pub const PIPE_TOKEN : usize = 0;

pub struct TaskBookkeeping {
    pub parent: Option<SharedTask>, // Arc synchronisation necessary
    pub children: Vec<usize>, // Cast as *const Task, memory safe it is because never dereferenced
    pub producer : usize, // Cast as *const State, memory safe because pinned
}

pub struct Task {
    pub future: RefCell<ExecutorFuture>,
    pub b : Mutex<TaskBookkeeping>,
}

impl Task {
    pub fn new(future : ExecutorFuture, parent : Option<SharedTask>, producer : *const State) -> Self {
        Self {  
            future: RefCell::new(future),
            b : Mutex::new(TaskBookkeeping { 
                parent, 
                children : Vec::new(),
                producer : producer as usize ,
            })
        }
    }

    pub fn join_parent(self : Arc<Self>) {
        let mut book = self.b.lock().unwrap();
        let execution_cx = unsafe { &*(book.producer as *const State) }; // state is pinned
        let mut parent_waiting = false;
        let task_parent = book.parent.take();
        {
            if let Some(ref parent_book_locked) = task_parent {
                let mut parent_book = parent_book_locked.b.lock().unwrap();
                let len = parent_book.children.len();
                parent_book.children.retain(|child| {
                    self.as_ref() as *const Self != *child as *const Self
                });
                
                if len == parent_book.children.len() {
                    // this task was not removed from the parent, i.e. the parent is waiting
                    parent_waiting = true;
                }
            }
        }
        if parent_waiting {
            execution_cx.push(task_parent.unwrap());
        }
        if execution_cx.is_empty() {
            execution_cx.notify();
        }
    }

    pub fn task_id(arc : &Arc<Self> ) -> usize {
        arc.as_ref() as *const Self as usize // logically pinned, todo: pin
    }
}

pub fn current_task() -> SharedTask {
    CURRENT.with(|x| { x.borrow().as_ref().unwrap().upgrade()}).unwrap()
}

thread_local! {
    pub static CURRENT: RefCell<Option<WeakTask>> = RefCell::new(None);
}