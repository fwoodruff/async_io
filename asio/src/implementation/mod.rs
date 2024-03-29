

pub(crate) mod state;
pub(crate) mod task;
pub(crate) mod futures;
pub(crate) mod executor;

use crate::JoinFuture;

use self::state::blockingstate::State;
use self::executor::Executor;
use self::task::*;

use std::{
    future::Future,
    sync::Arc,
    pin::Pin,
};

// Allows users to fork new task from existing async functions
pub(crate) fn async_spawn_impl(f: impl Future<Output = ()> + Send + 'static) -> JoinFuture {
    let current_task = current_task();
    let locking_cx: &State;
    let shared_new_task : SharedTask;
    {  
        let myref = current_task.as_ref();
        let mut task_books = myref.b.lock().unwrap();
        let new_task = Task::new(Box::pin(f), Some(current_task.clone()), task_books.producer as *const State );
        shared_new_task = Arc::pin(new_task);
        task_books.children.push(shared_new_task.task_id());
        // State is a member of an executor, and executors are always pinned
        locking_cx = unsafe { &*( task_books.producer as *const State) };
    }
    let ptr = shared_new_task.task_id();
    locking_cx.fork(shared_new_task);
    JoinFuture::new(ptr)
}

 // Entry point for user-provided async main
pub(crate) fn runtime_impl(main_task: impl Future<Output = ()> + Send + 'static) {
    let e = Executor::new(main_task);
    Pin::new(&e).run_main();
}