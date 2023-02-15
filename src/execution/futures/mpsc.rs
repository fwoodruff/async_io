use std::{  task::{Poll, Context}, sync::{mpsc::{self, *}, 
            Arc, atomic::{Ordering, AtomicBool}}, future::Future, cell::RefCell};

use crate::execution::{task::{SharedTask, current_task}, state::taskqueue::current_state};

// This builds an async mpsc queue over a blocking mpsc queue



struct ReceiverImpl<T> {
    receiver : mpsc::Receiver<T>,
    wakeable : AtomicBool,
    resumable : RefCell<Option<SharedTask>>,
}

unsafe impl<T: Send> Send for ReceiverImpl<T> { }
unsafe impl<T: Send> Sync for ReceiverImpl<T> { }

impl<T> ReceiverImpl<T> {
    fn new(receiver : mpsc::Receiver<T>) -> Self {
        Self { 
            receiver,
            wakeable : AtomicBool::new(false),
            resumable : RefCell::new(None)
        } 
    }
}

pub struct AsyncReceiver<T> {
    shared_receiver : Arc<ReceiverImpl<T>>,
}

impl<T> AsyncReceiver<T> {
    fn new(shared_receiver : Arc<ReceiverImpl<T>>) -> Self {
        Self {
            shared_receiver,
        }
    }

    pub fn try_receive(&self) -> Result<T, TryRecvError> {
        self.shared_receiver.receiver.try_recv()
    }

    pub fn receive(&self) -> AsyncReceiverFuture<'_, T> {
        AsyncReceiverFuture::new(self)
    }
}

pub struct AsyncReceiverFuture<'a, T> {
    receiver : &'a AsyncReceiver<T>,
}

impl<'a, T> AsyncReceiverFuture<'a, T> {
    fn new(receiver : &'a AsyncReceiver<T>) -> Self {
        Self {
            receiver,
        }
    }
}

impl< T> Future for AsyncReceiverFuture<'_, T> {
    type Output = Result<T, RecvError>;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let result = self.receiver.shared_receiver.receiver.try_recv();
        match result {
            Ok(res) => {
                Poll::Ready(Ok(res))
            }
            Err(_) => {
                let resumable = & self.receiver.shared_receiver.resumable;
                let old = resumable.borrow_mut().replace(current_task());
                if old.is_some() {
                    let failstate = self.receiver.shared_receiver.receiver.recv();
                    if failstate.is_ok() {
                        panic!();
                    }
                    return Poll::Ready(failstate);
                }
                self.receiver.shared_receiver.wakeable.store(true, Ordering::Release);
                Poll::Pending
            }
        }
    }
}

#[derive(Clone)]
pub struct AsyncSender<T> {
    sender : mpsc::Sender<T>,
    receiver : Arc<ReceiverImpl<T>>,
}


impl<T> AsyncSender<T> {
    fn new(sender : mpsc::Sender<T>, receiver : Arc<ReceiverImpl<T>>) -> Self {
        Self {
            sender,
            receiver
        }
    }

    pub fn send(&self, value : T) -> Result<(), SendError<T>> {
        self.sender.send(value)?;
        if self.receiver.wakeable.load(Ordering::Acquire) {
            let can_resume = self.receiver.wakeable.swap(false, Ordering::AcqRel);
            if can_resume {
                let mut resumable = self.receiver.resumable.borrow_mut();
                let state = unsafe { current_state() } ;
                let task = resumable.take().unwrap();
                state.push(task);
            }
        }
        Ok(())
    }
}


pub fn async_channel<T>() -> (AsyncSender<T>, AsyncReceiver<T>) {
    let (sender, receiver) = channel();
    let receiver_impl = Arc::new(ReceiverImpl::new(receiver));
    let receiver_clone = receiver_impl.clone();
    (AsyncSender::new(sender, receiver_impl), AsyncReceiver::new(receiver_clone))
}
