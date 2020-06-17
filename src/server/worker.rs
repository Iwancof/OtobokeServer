#![allow(unused)]

use std::{
    sync::{
        Arc,
        Mutex,
        mpsc::{
            Sender,
            SyncSender,
            SendError,
            Receiver,
            channel,
            RecvTimeoutError,
            sync_channel,
        },
    },
    clone::{
        Clone,
    },
    time::{
        Duration,
    },
    thread,
    process::exit
};

pub struct Worker {
    task: Box<dyn Fn() -> WorkerResult + Send + Sync>,
    send_to_logger: SyncSender<String>,
    // Worker: Send + Sync.
}
pub enum WorkerResult {
    Complete,  // Not system damage(may be error but not catch. use shared pointer)
    Pollution(String), // System damage 
}
impl Worker {
    pub fn new(task: Box<dyn Fn() -> WorkerResult + Send + Sync>, instance_sender: SyncSender<String>) -> Self {
        Self {
            task: task,
            send_to_logger: instance_sender,
        }
    }
    pub fn run(&self) {
        match (self.task)() {
            WorkerResult::Complete => {
                // Complate task
                println!("Complete task");
            },
            WorkerResult::Pollution(message) => {
                // Worker detect critical damage to instance.
                self.report(message);
            }
        }
    }
    fn report(&self, msg: String) -> ! {
        match self.send_to_logger.send(msg) {
            Ok(_) => {
                // Suggest killing this instance to observer
                println!("This instance will be killed");
            },
            Err(_) => {
                println!("Could not send to observer. ");
                println!("Critical damage detected"); 
                println!("System will be abort"); 
                exit(-1);
            }
        }
        loop {
        }
    }
}

#[test]
fn worker_success_test() {
    let (sender, receiver) = sync_channel(16);
    let worker = Worker::new(
        Box::new(move || {
            thread::sleep(Duration::from_millis(100));
            WorkerResult::Complete
        }),sender.clone(),);
    thread::spawn(move || {
        worker.run();
    });
    assert_eq!(
        receiver.recv_timeout(Duration::from_millis(200)),
        Err(RecvTimeoutError::Timeout),
    );
}

#[test]
fn worker_fail_test() {
    let (sender, receiver) = sync_channel(16);
    let worker = Worker::new(
        Box::new(move || {
            thread::sleep(Duration::from_millis(100));
            WorkerResult::Pollution("Some error".to_string())
        }), sender,);
    thread::spawn(move || {
        worker.run();
    });
    assert_eq!(
        receiver.recv_timeout(Duration::from_millis(200)),
        Ok("Some error".to_string())
    );
}

