
use super::{ State, Task, CURRENT, NUMTHREADS, task::SharedTask};
use std::{
    task::{ 
        Waker, 
        Context, 
        Wake
    },
    sync::Arc,
    future::Future,
    pin::Pin,
};

use pin_weak::sync::PinWeak;

pub(super)
struct Executor {
    execution_context: State,
}

impl Executor {
    // Creates a new executor with the async entry point
    pub(super)
    fn new(async_start: impl Future<Output = ()> + Send + 'static) -> Self {
        let result = Self {
            execution_context: State::new(),
        };
        let task = Task::new(Box::pin(async_start), None, &result.execution_context);
        let shared_task = Arc::pin(task);
        result.execution_context.push(shared_task);
        result
    }

    // Runs a task up to its next suspend point on the current thread
    // Async is conceptually hard so it's worth note that when a parent
    // future awaits a child future that returns ready, the parent
    // future does not then need to suspend, and this function is *not* called.
    fn poll_task(self : Pin<&Self>, some_task : SharedTask) {
        let wk = PinWeak::downgrade(some_task.clone());
        CURRENT.with(|x| { x.replace(Some(wk)); });
        let mut fut = some_task.future.borrow_mut();
        let noopwaker = Waker::from(Arc::new(NoopWaker::new()));
        let mut cx = Context::from_waker(&noopwaker);
        let res = fut.as_mut().poll(&mut cx);
        drop(fut);
        match res {
            std::task::Poll::Ready(_) => {
                self.execution_context.join(some_task);
            },
            std::task::Poll::Pending => {},
        }
    }

    // If there are no tasks queued for execution, and polling for new tasks yields nothing
    // Then wait
    fn no_task(self : Pin<&Self>) {
        match self.execution_context.poll() {
            Some(woken_tasks) => self.push_woken(woken_tasks),
            None => self.execution_context.wait(),
        }
    }

    // Polling yielded a few tasks so we should add them to the executor's work queue.
    fn push_woken(self : Pin<&Self>, woken_tasks: Vec<SharedTask>) {
        for task in woken_tasks.into_iter() {
            self.execution_context.push(task);
        }
    }

    fn thread_loop(self : Pin<&Self>) -> bool {
        let res = !self.execution_context.is_empty();
        if res {
            match self.execution_context.pop() {
                Some(some_task) => self.poll_task(some_task),
                None => self.no_task()
            }
        }
        res
    }

    fn run_thread(self : Pin<&Self>) {
        while self.thread_loop() { }
    }

    // Start the thread pool and set the user code running
    pub(super) fn run_main(self : Pin<&Self>) {
        let mut threads : Vec<std::thread::JoinHandle<()>> = Vec::new();
        for _ in 0..NUMTHREADS {
            self.push_one(&mut threads);
        }
        for thd in threads.into_iter() {
            thd.join().expect("failed to join thread");
        }
    }

    // Add a thread to the thread pool and set to work
    fn push_one(self : Pin<&Self>, threads: &mut Vec<std::thread::JoinHandle<()>>) {
        let exec = self.get_ref() as *const Executor as usize;
        let thd = std::thread::spawn(move || {
            let thread_exec = unsafe { Pin::new(&*(exec as *const Executor)) };
            // Executor is pinned, so ok to Send a pointer to it
            thread_exec.run_thread();
        });
        threads.push(thd);
    }
}

// Wakers are part of the polling interface.
// I am not using this interface.
struct NoopWaker;

impl NoopWaker {
    fn new() -> Self { Self {} }
}

impl Wake for NoopWaker {
    fn wake(self: Arc<Self>) { }
}