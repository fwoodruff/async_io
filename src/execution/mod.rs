

pub mod state;
pub mod task;
pub mod futures;
pub mod executor;

use self::state::State;
use self::executor::Executor;
use self::futures::{*, join::*};
use self::task::*;
use std::{
    future::Future,
    io::Read,
    sync::Arc,
    pin::Pin,
    cell::RefCell,
};
use mio::net::TcpStream;

pub fn async_spawn(f: impl Future<Output = ()> + Send + 'static) -> join::JoinFuture {
    let current_task = current_task();
    let locking_cx: &State;
    let shared_new_task : Arc<Task>;
    {
        let mut task_books = current_task.as_ref().b.lock().unwrap();
        let new_task = Task::new(Box::pin(f), Some(current_task.clone()), task_books.producer as *const State );
        shared_new_task = Arc::new(new_task);
        task_books.children.push(shared_new_task.as_ref() as *const Task as usize);
        locking_cx = unsafe { &*( task_books.producer as *const State) }; // State is pinned for all executors
    }
    let ptr = shared_new_task.as_ref() as *const Task;
    locking_cx.spawn(shared_new_task);
    JoinFuture::new(ptr)
}

pub fn runtime(main_task: impl Future<Output = ()> + Send + 'static) {
    let e = Executor::new(main_task);
    Pin::new(&e).run_main();
}