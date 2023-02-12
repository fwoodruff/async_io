
use std::{sync::{atomic::*, Mutex}, collections::VecDeque, future::Future, pin::Pin, ops::DerefMut, cell::UnsafeCell};
use crate::execution::{task::{SharedTask, current_task}, state::execute::current_state};
use std::ops::Deref;

pub struct AsyncMutex<T> {
    lock : AsyncMutexInternal,
    data : UnsafeCell<T>,
}


impl<'a, T> AsyncMutex<T> {
    pub fn new(value : T) -> Self {
        Self {
            lock : AsyncMutexInternal::new(),
            data : UnsafeCell::new(value),
        }
    }

    pub async fn lock(&'a self) -> AsyncGuard<'a, T> {
        let fut = self.lock.lock_internal();
        fut.await;
        AsyncGuard::new(self)
    }
}

unsafe impl<T: Send> Send for AsyncMutex<T> { }
unsafe impl<T: Sync> Sync for AsyncMutex<T> { }

unsafe impl<'a, T: Sync> Sync for AsyncGuard<'a, T> { }


pub struct AsyncGuard<'a, T : 'a> {
    data_ref : &'a AsyncMutex<T>,
}

impl<'a ,T> AsyncGuard<'a, T> {
    fn new(data_ref : &'a AsyncMutex<T>) -> Self {
        Self {
            data_ref,
        }
    }
}

impl<'a, T : 'a> Drop for AsyncGuard<'a, T> {
    fn drop(&mut self) {
        self.data_ref.lock.unlock_internal();
    }
}

impl<'a, T> Deref for AsyncGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.data_ref.data.get() }
    }
}

impl<'a, T> DerefMut for AsyncGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data_ref.data.get() }
    }
}


struct AsyncMutexInternal {
    atom : AtomicBool,
    suspended : Mutex<VecDeque<SharedTask>>,
}

impl AsyncMutexInternal {
    fn new() -> Self {
        AsyncMutexInternal { 
            atom : AtomicBool::new(false),
            suspended : Mutex::new(VecDeque::new()),
        }
    }

    fn lock_internal<'a>(&'a self) -> LockFuture<'a> {
        LockFuture { mutex_ref : self  }
    }

    fn unlock_internal(&self) {
        let local_task : Option<SharedTask>;
        {
            let mut lk = self.suspended.lock().unwrap();
            local_task = lk.pop_back();
        }
        self.atom.store(false, Ordering::SeqCst);
        let state = unsafe { current_state() } ;
        if let Some(task) = local_task {
            state.push(task);
        }
    }
}

struct LockFuture<'a> {
    mutex_ref : &'a AsyncMutexInternal,
}

unsafe impl<'a> Send for LockFuture<'a> { }


impl<'a> Future for LockFuture<'a> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let old = self.mutex_ref.atom.swap(true, Ordering::SeqCst);
        if old {
            let task = current_task();
            let mut task_list = self.mutex_ref.suspended.lock().unwrap();
            task_list.push_front(task);
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(())
        }
    }
}