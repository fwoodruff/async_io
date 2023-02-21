
use std::{
    sync::{Mutex, MutexGuard},
    cell::RefCell,
    collections::HashMap,
    io::{
        Read,
        Write
    }
};
use mio::Interest;
use super::super::PIPE_TOKEN;
use crate::implementation::SharedTask;
use std::thread;


pub(crate) enum NetFD {
    // Raw pointers because tasks may temporarily live in thread local memory so
    // hard to prove lifetimes are bounded at compile time
    Listener(*mut mio::net::TcpListener),
    Stream(*mut mio::net::TcpStream),
}

struct RegisteredTask {
    task : SharedTask,
    descriptor : NetFD,
}

impl RegisteredTask {
    fn new(task: SharedTask, descriptor : NetFD) -> Self {
        Self {
            task,
            descriptor,
        }
    }
}

pub(crate)
struct SharedPoller {
    read_socket : Mutex<RefCell<mio::net::UnixStream>>,
    write_socket : Mutex<mio::net::UnixStream>,
    unique_access : Mutex<bool>,
    locked_poller : Mutex<mio::Poll>,
    token_map : Mutex<HashMap<mio::Token, RegisteredTask >>,
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
    fn force_lock(&self) -> MutexGuard<mio::Poll> {
        loop {
            let mut buff = [0u8; 1];
            while self.write_socket.lock().unwrap().write(&buff[..]).is_ok() {}

            let poller_guard = self.locked_poller.try_lock();
            let mut read_sock_cell = self.read_socket.lock().unwrap();
            let read_sock = read_sock_cell.get_mut();

            while read_sock.read(&mut buff[..]).is_ok() {}
            
            if poller_guard.is_err() {
                thread::yield_now();
                continue;
            }
            return poller_guard.unwrap();
        }
    }
    
    // Register a task which can be woken up when network IO completes
    pub(in crate::implementation)
    fn register(&self, source : NetFD, task : SharedTask, interests : Interest) {
        let poller = self.force_lock();
        let token = mio::Token(task.task_id());
        let _res = match source {
            NetFD::Listener(listener) => poller.registry().register(unsafe{&mut *listener}, token, interests),
            NetFD::Stream(stream) => poller.registry().register(unsafe{&mut *stream}, token, interests),
        };
        let rtask = RegisteredTask::new(task, source);
        let old_value = self.token_map.lock().unwrap().insert(token, rtask);
        
        if old_value.is_some() {
            panic!();
        }
    }

    // List all the tasks that can resume
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
        if pollres.is_err() {
            return Some(woken_tasks);
        }
        for event in events.into_iter() {
            let tt = event.token();
            if tt.0 == PIPE_TOKEN {
                continue;
            }
            
            let reg_task = self.token_map.lock().unwrap().remove(&tt).unwrap();
            woken_tasks.push(reg_task.task);
            
            let _res = match reg_task.descriptor {
                NetFD::Listener(listener) => local_poll.registry().deregister(unsafe{&mut *listener}),
                NetFD::Stream(stream) => local_poll.registry().deregister(unsafe{&mut *stream}),
            };
        }
        Some(woken_tasks)
    }

    // nudge the poller so it returns with some or no tasks
    pub(super)
    fn notify(&self) {
        let _unused = self.force_lock();
    }

}
