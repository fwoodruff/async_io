

use crate::JoinFuture;
use crate::implementation::task::current_task;
use crate::implementation::task::TaskID;

use std::task::Poll;
use std::{
    task::Context,
    pin::Pin,
    future::Future,
};



impl JoinFuture {
    pub(crate) fn new(child : TaskID) -> Self {
        Self {
            child,
        }
    }
}

impl Future for JoinFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let current_task = current_task();
        let mut current_book = current_task.b.lock().unwrap();
        let len = current_book.children.len();
        current_book.children.retain(|tchild| {
            let book_child = tchild;
            let future_child = self.child;
            future_child != *book_child
        });
        if len == current_book.children.len() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
