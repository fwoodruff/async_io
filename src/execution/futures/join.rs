

use crate::execution::task::current_task;
use crate::execution::task::TaskID;

use std::{
    task::Context,
    pin::Pin,
    future::Future,
};

pub struct JoinFuture {
    child : TaskID
}

impl JoinFuture {
    pub(in crate::execution) fn new(child : TaskID) -> Self {
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
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}
