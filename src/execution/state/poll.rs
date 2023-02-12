
use std::{
    sync::Mutex,
    cell::RefCell,
    collections::HashMap,
    io::{
        Read,
        Write
    }
};
use mio::Interest;

use super::{ 
    SharedTask,
    super::PIPE_TOKEN,
};
use std::thread;

pub
struct SharedPoller {
    read_socket : Mutex<RefCell<mio::net::UnixStream>>,
    write_socket : Mutex<mio::net::UnixStream>,
    unique_access : Mutex<bool>,
    locked_poller : Mutex<mio::Poll>,
    token_map : Mutex<HashMap<mio::Token, SharedTask>>,
    // poll never reads from a pipe, and will always unblock the poll if a registration is attempted
}

impl SharedPoller {
    // Tasks that need to wait on network IO register themselves with a SharedPoller object
    // Polling yields tasks that can commence with their IO operations
    pub(super)
    fn new() -> Self {
        let (read_socket, write_socket) = mio::net::UnixStream::pair().expect("Couldn't create a pipe");
        let res = Self {
            read_socket : Mutex::new(RefCell::new(read_socket)),
            write_socket : Mutex::new(write_socket),
            locked_poller : Mutex::new(mio::Poll::new().unwrap()),
            token_map : Mutex::new(HashMap::new()),
            unique_access : Mutex::new(false),
        };
        
        {
            let poller = res.locked_poller.lock().unwrap();
            let mut rs = res.read_socket.lock().unwrap();
            let v = rs.get_mut();
            poller.registry().register(v, mio::Token(PIPE_TOKEN), Interest::READABLE).unwrap();
        }
        res
    }

    // If a poller is blocked on polling, we need to unblock so we can update with new tasks
    // otherwise we're just locking a mutex
    fn force_lock(&self) -> std::sync::MutexGuard<mio::Poll> {
        loop {
            let mut buff = [0u8; 1];
            while self.write_socket.lock().unwrap().write(&buff[..]).expect("failed to write byte to pipe") != 1 {
                eprintln!("reloop");
            }
            let res = self.locked_poller.try_lock();
            let mut rr = self.read_socket.lock().unwrap();
            let bb = rr.get_mut();
            while  bb.read(&mut buff[..]).unwrap() != 1 {
                eprintln!("reloop");
            }
            if let Err(_) = res {
                thread::yield_now();
                continue;
            }
            return res.unwrap();
        }
        
    }
    
    // Register a task which can be woken up when network IO completes
    pub(in crate::execution)
    fn register(&self, source : &mut impl mio::event::Source, task : SharedTask, interests : Interest) {
        
        let poller = self.force_lock();
        
        let token = mio::Token(task.task_id());
        let old_value = self.token_map.lock().unwrap().insert(token, task);
        
        
        if let Some(_value) = old_value {
            assert!(false);
        }
        
        let _res = poller.registry().register(source, token, interests);
        
    }

    // remove a task so that it is no longer waiting for network IO
    pub(in crate::execution)
    fn deregister(&self, source : &mut impl mio::event::Source, _task : SharedTask) {
        let local_poll = self.force_lock();
        let _res = local_poll.registry().deregister(source);
    }

    // List all the tasks that can commence
    pub(super)
    fn get_network_tasks(&self) -> Option<Vec<SharedTask>> {
        let access = self.unique_access.try_lock();
        if let Err(_error) = access {
            return None;
        }
        if self.token_map.lock().unwrap().len() == 0 {
            return None;
        }
        let mut local_poll = self.locked_poller.lock().unwrap();
        let mut events = mio::Events::with_capacity(1024);
        let pollres = local_poll.poll(&mut events, None);
        let mut woken_tasks = vec!();
        if let Err(_) = pollres {
            return Some(woken_tasks);
        }
        for event in events.into_iter() {
            let tt = event.token();
            if tt.0 == PIPE_TOKEN {
                continue;
            }
            
            
            let opt_task = self.token_map.lock().unwrap().remove(&tt);
            if let Some(task) = opt_task {
                // This subtly terrible if statement introduces short bursts of no-op syscall hot-loops.
                // The alternative is rewriting Mio to let me deregister arbitrary file descriptors
                // from the kqueue/epoll context because Mio::Poll decided to use both static dispatch
                // and inheritance rather than composition for Sources.
                woken_tasks.push(task);
            }

            
        }
        Some(woken_tasks)
    }

    // nudge the poller so it returns with some or no tasks
    pub(super)
    fn notify(&self) {
        let _unused = self.force_lock();
    }

    // checks if there are any tasks registered
    pub(super)
    fn is_empty(&self) -> bool {
        let tokens = self.token_map.lock().unwrap();
        tokens.is_empty()
    }
}
