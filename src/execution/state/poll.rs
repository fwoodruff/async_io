
use std::{
    sync::Mutex,
    cell::RefCell,
    collections::HashMap,
    io::{
        Read,
        Write
    }
};
use super::{ 
    SharedTask,
    super::PIPE_TOKEN,
    Task
};
use mio::Interest;

pub struct SharedPoller {
    read_socket : Mutex<RefCell<mio::net::UnixStream>>,
    write_socket : Mutex<mio::net::UnixStream>,
    single_access : Mutex<bool>,
    locked_poller : Mutex<mio::Poll>,
    token_map : Mutex<HashMap<mio::Token, SharedTask>>,
    // poll never reads from a pipe, and will always unblock the poll if a registration is attempted
}

impl SharedPoller {
    pub fn new() -> Self {
        let (read_socket, write_socket) = mio::net::UnixStream::pair().unwrap();
        let res = Self {
            read_socket : Mutex::new(RefCell::new(read_socket)),
            write_socket : Mutex::new(write_socket),
            locked_poller : Mutex::new(mio::Poll::new().unwrap()),
            token_map : Mutex::new(HashMap::new()),
            single_access : Mutex::new(false),
        };
        {
            let poller = res.locked_poller.lock().unwrap();
            let mut rs = res.read_socket.lock().unwrap();
            let v = rs.get_mut();
            poller.registry().register(v, mio::Token(PIPE_TOKEN), Interest::READABLE).unwrap();
        }
        res
    }

    fn force_lock(&self) -> std::sync::MutexGuard<mio::Poll> {
        let mut buff = [0u8; 1];
        while self.write_socket.lock().unwrap().write(&buff[..]).unwrap() != 1 {
            eprintln!("reloop");
        }
        let res = self.locked_poller.lock().unwrap();
        let mut rr = self.read_socket.lock().unwrap();
        let bb = rr.get_mut();
        while  bb.read(&mut buff[..]).unwrap() != 1 {
            println!("reloop");
        }
        res
    }

    pub fn register(&self, source : &mut impl mio::event::Source, task : SharedTask, interests : Interest) {
        let poller = self.force_lock();
        let token = mio::Token(Task::task_id(&task));
        self.token_map.lock().unwrap().insert(token, task);
        let _res = poller.registry().register(source, token, interests);
    }

    pub fn deregister(&self, source : &mut impl mio::event::Source) {
        let poller = self.force_lock();
        let _res = poller.registry().deregister(source);
    }

    pub fn poll(&self) -> Option<Vec<SharedTask>> {
        let access = self.single_access.try_lock();
        if let Err(_error) = access {
            return None;
        }
        if self.token_map.lock().unwrap().len() == 0 {
            return None;
        }
        let mut poll = self.locked_poller.lock().unwrap();
        let mut events = mio::Events::with_capacity(1024);
        poll.poll(&mut events, None).unwrap();
        let mut res = vec!();
        for event in events.into_iter() {
            let tt = event.token();
            if tt.0 == PIPE_TOKEN {
                continue;
            }
            let opt_task = self.token_map.lock().unwrap().remove(&tt).unwrap();
            res.push(opt_task);
        }
        Some(res)
    }

    pub fn notify(&self) {
        let _ = self.force_lock();
    }

    pub fn is_empty(&self) -> bool {
        let tokens = self.token_map.lock().unwrap();
        tokens.is_empty()
    }
}
