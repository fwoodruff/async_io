use std::{sync::atomic::{AtomicUsize, Ordering}, future::Future};



struct RingBuffer<T : Default> {
    buffer : Vec<T>,
    num_elems : AtomicUsize,
}

impl<'a, T > RingBuffer<T> where T : Clone + Default {
    pub fn new(bound : usize) -> Self {
        let mut buffer : Vec<T> = vec!();
        buffer.resize(bound, Default::default());
        Self {
            buffer,
            num_elems : AtomicUsize::new(0),
        }
    }

    pub fn send(&'a self, value : T) -> RingSendFuture<T> {
        RingSendFuture::new(self, value)
    }

    pub fn recv(&'a self) -> RingRecvFuture<T> {
        RingRecvFuture::new(self)
    }
}

struct RingSendFuture<'a, T : Default> {
    buffer : &'a RingBuffer<T>,
    value : T,
}

impl<'a, T : Default> RingSendFuture<'a, T> {
    fn new(buffer : &'a RingBuffer<T>, value : T) -> Self {
        Self {
            buffer,
            value,
        }
    }
}

impl<'a, T : Default> Future for RingSendFuture<'a, T> {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut elems = self.buffer.num_elems.load(Ordering::Acquire);
        while elems < self.buffer.buffer.len() {
            // CAS and take an element
            
        }
        todo!()
    }
}

struct RingRecvFuture<'a, T : Default> {
    buffer : &'a RingBuffer<T>
}

impl<'a, T : Default> RingRecvFuture<'a, T> {
    fn new(buffer : &'a RingBuffer<T>) -> Self {
        Self {
            buffer,
        }
    }
}

impl<'a, T : Default> Future for RingRecvFuture<'a, T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        todo!()
    }
}