
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
    pub parent: Option<SharedTask>, // Arc synchronisation necessary
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
        let lhs = Arc::into_raw(Pin::into_inner(self.clone()));
        lhs as TaskID
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