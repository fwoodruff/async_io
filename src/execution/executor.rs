
use super::{ State, Task, CURRENT, NUMTHREADS};
use std::{
    task::{ 
        Waker, 
        Context, 
        Wake
    },
    sync::Arc,
    future::Future,
    pin::Pin
};

pub(super) struct Executor {
    execution_context: State,
}

impl Executor {
    
    pub(super) fn new(async_start: impl Future<Output = ()> + Send + 'static) -> Self {
        let result = Self {
            execution_context: State::new(),
        };
        let task = Task::new(Box::pin(async_start), None, &result.execution_context);
        let shared_task = Arc::new(task);
        result.execution_context.spawn(shared_task);
        result
    }

    fn poll_task(&self, some_task : Arc<Task>) {
        let wk = Arc::downgrade(&some_task);
        CURRENT.with(|x| { x.replace(Some(wk)); });
        let mut fut = some_task.future.borrow_mut();
        let noopwaker = Waker::from(Arc::new(NoopWaker::new()));
        let mut cx = Context::from_waker(&noopwaker);
        let res = fut.as_mut().poll(&mut cx);
        drop(fut);
        match res {
            std::task::Poll::Ready(_) => {
                some_task.join_parent();
            },
            std::task::Poll::Pending => {},
        }
    }

    

    fn no_task(&self) {
        match self.execution_context.poll() {
            Some(woken_tasks) => self.push_woken(woken_tasks),
            None => self.execution_context.wait(),
        }
    }

    fn push_woken(&self, woken_tasks: Vec<Arc<Task>>) {
        for task in woken_tasks.into_iter() {
            self.execution_context.push(task);
        }
    }

    fn thread_loop(&self) {
        //todo!("try lock poll"); // I can then claim that this is lock free whenever there is anything to do
        if self.execution_context.is_empty() { return; }
        match self.execution_context.pop() {
            Some(some_task) => self.poll_task(some_task),
            None => self.no_task()
        }
    }

    fn run_thread(&self) {
        loop {
            self.thread_loop();
        }
    }

    pub(super) fn run_main(self : Pin<&Self>) {
        let mut threads : Vec<std::thread::JoinHandle<()>> = Vec::new();
        for _ in 0..NUMTHREADS {
            self.push_one(&mut threads);
        }
        for thd in threads.into_iter() {
            thd.join().expect("failed to join thread");
        }
    }

    fn push_one(self : Pin<&Self>, threads: &mut Vec<std::thread::JoinHandle<()>>) {
        let exec = self.get_ref() as *const Executor as usize;
        let thd = std::thread::spawn(move || {
            let cc = unsafe { &*(exec as *const Executor) }; // Executor is pinned, so ok to Send a pointer to it
            cc.run_thread();
        });
        threads.push(thd);
    }
}

struct NoopWaker;

impl NoopWaker {
    fn new() -> Self { Self {} }
}

impl Wake for NoopWaker {
    fn wake(self: Arc<Self>) { }
}