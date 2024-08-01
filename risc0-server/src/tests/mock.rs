use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use crate::start_grpc_server;

pub struct Server {
    pub started: AtomicBool,
}

impl Server {
    pub fn new() -> Server {
        Server {
            started: AtomicBool::new(false),
        }
    }

    pub async fn init_server(&mut self) {
        if !self.started.load(Ordering::Relaxed) {
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("runtime starts");
                rt.spawn(start_grpc_server("0.0.0.0:14001"));
                loop {
                    thread::sleep(Duration::from_millis(100_000));
                }
            });
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.started.store(true, Ordering::Relaxed);
        }
    }
}
