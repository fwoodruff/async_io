
use super::super::*;
use crate::execution::state::execute::current_state;

pub struct WriteFuture<'a> {
    buff : &'a [u8],
    stream : &'a RefCell<TcpStream>,
    registered : bool,
}

unsafe impl Send for WriteFuture<'_> {} // future has a single owner when dropped or polled

impl<'a> WriteFuture<'a> {
    pub fn new(buff : &'a [u8], stream : &'a RefCell<TcpStream>) -> Self {
        Self {
            buff,
            stream,
            registered : false,
        }
    }
}

#[macro_export]
macro_rules! readwriteresult {
    ( $res:expr, $slef:expr, $stream_ref:expr ) => {
        let state_ptr = current_state();
        let state = unsafe { &mut *state_ptr };
            match $res {
                Ok(_) => {
                    if $slef.registered {
                        state.pl.deregister($stream_ref);
                        $slef.registered = false;
                    }
                    return std::task::Poll::Ready($res);
                }
                Err(ref error) => {
                    match error.kind() {
                        std::io::ErrorKind::WouldBlock => {
                            if !$slef.registered {
                                state.pl.register($stream_ref, current_task(), mio::Interest::READABLE);
                                $slef.registered = true;
                            }
                            return std::task::Poll::Pending;
                        }
                        std::io::ErrorKind::Interrupted => {
                            continue;
                        }
                        _ => {
                            if $slef.registered {
                                state.pl.deregister($stream_ref);
                                $slef.registered = false;
                            }
                            return std::task::Poll::Ready($res);
                        }
                    }
                }
            }
            
    };
}


impl Future for WriteFuture<'_> {
    type Output = Result<usize, std::io::Error>;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            let mut stream_borrow = self.stream.borrow_mut();
            let mut stream_ref = stream_borrow.by_ref();
            let res = std::io::Write::write(&mut stream_ref, self.buff);
            readwriteresult!(res, self, stream_ref);
        }
    }
}