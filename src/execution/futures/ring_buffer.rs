use std::{sync::atomic::{AtomicUsize, Ordering}, future::Future, cell::RefCell};

use std::task::Poll;

pub struct RingBuffer<T> {
    buffer : Vec<RefCell<Option<T>>>,
    num_elems : AtomicUsize,
    insertion_idx : AtomicUsize,
    removal_idx : AtomicUsize,
}

unsafe impl<T: Send> Send for RingBuffer<T> { }
unsafe impl<T: Send> Sync for RingBuffer<T> { }

impl<T : Clone> RingBuffer<T> {
    pub fn new(bound : usize) -> Self {
        
        let mut buffer : Vec<RefCell<Option<T>>> = vec!();
        buffer.resize(bound, RefCell::new(None));
        
        Self {
            buffer,
            num_elems : AtomicUsize::new(0),
            insertion_idx : AtomicUsize::new(0),
            removal_idx : AtomicUsize::new(0),
        }
    }

    pub fn send(&self, value : T) -> RingSendFuture<T> {
        RingSendFuture::new(self, value)
    }

    pub fn recv(&self) -> RingRecvFuture<T> {
        RingRecvFuture::new(self)
    }
}

pub struct RingSendFuture<'a, T> {
    buffer : &'a RingBuffer<T>,
    value : RefCell<Option<T>>,
}

impl<'a, T> RingSendFuture<'a, T> {
    fn new(buffer : &'a RingBuffer<T>, value : T) -> Self {
        Self {
            buffer,
            value : RefCell::new(Some(value)),
        }
    }
}

impl<T> Future for RingSendFuture<'_, T> {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let elements = &self.buffer.num_elems;
        let mut elems = elements.load(Ordering::Acquire);

        while elems < self.buffer.buffer.len() {
            match elements.compare_exchange_weak(elems, elems + 1, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => {
                    let idx = self.buffer.insertion_idx.fetch_add(1, Ordering::AcqRel);
                    let value = self.value.take();
                    self.buffer.buffer[idx % self.buffer.buffer.len()].replace(value);
                    return Poll::Ready(());
                },
                Err(x) => elems = x,
            }
        }
        Poll::Pending
    }
}

pub struct RingRecvFuture<'a, T> {
    buffer : &'a RingBuffer<T>
}

impl<'a, T : 'a> RingRecvFuture<'a, T> {
    fn new(buffer : &'a RingBuffer<T>) -> Self {
        Self {
            buffer,
        }
    }
}

impl<T> Future for RingRecvFuture<'_, T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let elements = &self.buffer.num_elems;
        let mut elems = elements.load(Ordering::Acquire);
        while elems > 0 {
            let newval = elems - 1;
            match elements.compare_exchange_weak(elems, newval, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => {
                    let idx = self.buffer.removal_idx.fetch_add(1, Ordering::AcqRel);
                    let value = self.buffer.buffer[idx % self.buffer.buffer.len()].take();
                    return Poll::Ready(value.unwrap());
                },
                Err(x) => elems = x,
            }
        }
        Poll::Pending
    }
}