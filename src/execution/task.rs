
use std::pin::Pin;
use std::future::Future;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use super::*;
use pin_weak::sync::*;

type ExecutorFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
pub(super)
type SharedTask = Pin<Arc<Task>>;
type WeakTask = PinWeak<Task>;

pub const NUMTHREADS : usize = 2;
pub const PIPE_TOKEN : usize = 0;



pub(super)
type TaskID = usize;


pub struct TaskBookkeeping {
    parent: Option<SharedTask>, // Arc synchronisation necessary
    pub children: Vec<TaskID>,
    // When an async function spawns a child task, the child needs to determine its parent task
    // Async functions don't offer access to the running task directly. Passing the current task as an
    // argument would infect every user-provided async function.
    // The parent is instead accessed via thread local memory. Thread local memory has static lifetime,
    // which is longer than the lifetime of the executor. Tasks have a lifetime shorter than an
    // executor.  Hence, the state must be accessed via a pointer not a reference.
    // State lives on the stack so reference-counting is not desirable.
    pub producer : *const State,
}

pub struct Task {
    pub future: RefCell<ExecutorFuture>,
    pub b : Mutex<TaskBookkeeping>,
}

// Task objects execute Futures and contain bookkeeping to facilitate spawning and joining other tasks
impl Task {
    pub(super)
    fn new(future : ExecutorFuture, parent : Option<SharedTask>, producer : *const State) -> Self {
        Self {  
            future: RefCell::new(future),
            b : Mutex::new(TaskBookkeeping { 
                parent, 
                children : Vec::new(),
                producer : producer ,
            })
        }
    }

    // SharedTask objects are pinned so the inner pointer can be used as a unique identifier
    pub
    fn task_id(self : &SharedTask) -> TaskID {
        unsafe {
            let lhs = Arc::into_raw(Pin::into_inner_unchecked(self.clone()));
            lhs as TaskID
        }
    }
    

    // At the end of a task's execution, it must alert its parent task that it is done
    // If a parent task is waiting for this task, we must resume the parent task.
    // If anyone is wondering why this isn't part of drop(Task), note that 
    // we can often start the parent when there is more than one reference to the task
    pub(super)
    fn join_parent(self : SharedTask) {
        let mut book = self.b.lock().unwrap();
        let execution_cx = unsafe { Pin::new(&*(book.producer as *const State)) };
        let mut parent_waiting = false;
        let task_parent = book.parent.take();
        {
            if let Some(ref parent_book_locked) = task_parent {
                let mut parent_book = parent_book_locked.b.lock().unwrap();
                let len = parent_book.children.len();
                parent_book.children.retain(|child| {
                    self.task_id() != *child
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

    
    
}

// Retrieve the currently running task from thread local memory
pub(super)
fn current_task() -> SharedTask {
    CURRENT.with(|x| { x.borrow().as_ref().unwrap().upgrade()}).unwrap()
}

thread_local! {
    pub(super) static CURRENT: RefCell<Option<WeakTask>> = RefCell::new(None);
}