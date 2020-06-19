#![allow(unused)]
#![allow(dead_code)]

use std::{
    time::{
        Duration,
        Instant,
    },
    thread,
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering,
        },
        mpsc::{
            channel,
            Sender,
            Receiver,
            SyncSender,
            sync_channel,
        }
    },
    collections::{
        HashSet,
        LinkedList,
    }
};

use crate::server::worker::{
    Worker,
    WorkerResult,
};

// for test use
#[allow(unused)]
use std::ops::{Deref, DerefMut, Add, Sub};
use std::cmp::{PartialOrd};

fn run_worker_async(worker: Worker, span: Duration, flag: Arc<AtomicBool>) {
    thread::Builder::new().name("run_worker_async methot".to_string()).spawn(move || {
        loop {
            if flag.load(Ordering::Relaxed) == false {
                // stoping
                continue;
            }
            worker.run(); // ignore worker's time.
            thread::sleep(span);
        }
    }).unwrap();
}

pub struct WorkerConductor {
    worker_sender: SyncSender<String>, // worker to system.
    start_flag: Arc<AtomicBool>,
}
impl WorkerConductor {
    pub fn new(sys_sender: SyncSender<String>) -> Self {
        Self {
            worker_sender: sys_sender,
            start_flag: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn add_worker(&self, worker: Worker, span: Duration) {
        run_worker_async(worker, span, self.start_flag.clone());
    }
    pub fn add_task(&self, task: Box<dyn Fn() -> WorkerResult + Send + Sync>, span: Duration) {
        self.add_worker(Worker::new(task, self.worker_sender.clone()), span);
    }
    pub fn start(&self) {
        self.start_flag.store(true, Ordering::Relaxed);
    }
    pub fn stop(&self) {
        self.start_flag.store(false, Ordering::Relaxed);
    }
}

#[test]
fn timer_worker_test() {
    let (ss, rc) = sync_channel(16);
    let cond = WorkerConductor::new(ss);

    let task1_counter = Arc::new(Mutex::new(0));
    let cnt = task1_counter.clone();
    
    cond.add_task(Box::new(move || {
        *cnt.lock().unwrap() += 1;
        WorkerResult::Complete
    }), Duration::from_millis(100));

    let task2_counter = Arc::new(Mutex::new(0));
    let cnt = task2_counter.clone();
    
    cond.add_task(Box::new(move || {
        *cnt.lock().unwrap() += 1;
        WorkerResult::Complete
    }), Duration::from_millis(50));

    cond.start();
    thread::sleep(Duration::from_millis(1000));
    cond.stop();

    assert_eq!(2 * *task1_counter.lock().unwrap(), *task2_counter.lock().unwrap());
    assert!(*task1_counter.lock().unwrap() != 0);
}

pub fn time_task_reservation(task: impl(Fn() -> ()) + Send + 'static, span: Duration) -> Sender<Message> {
    let (s, r) = channel();

    thread::spawn(move || {
        match r.recv_timeout(span) {
            Ok(msg) => {
                match msg  {
                    Message::Stop => {
                        //println!("Cancell");
                        return; // stop this thread.
                    }
                }
            },
            Err(_) => {
                // there is no stop message. so we can do task.
                task();
            }
        }
    });
    s
}

pub enum Message {
    Stop,
}

#[test]
fn time_task_reservation_cancel_test() {
    let atomic_value = Arc::new(Mutex::new(0));
    let arc_clone = atomic_value.clone(); 
    let sender = time_task_reservation(move || {
        *arc_clone.lock().unwrap() = 1;
    }, Duration::from_millis(100));

    thread::sleep(Duration::from_millis(10));

    sender.send(Message::Stop);

    thread::sleep(Duration::from_millis(200));

    assert_eq!(0, *atomic_value.lock().unwrap());
}

#[test]
fn time_task_reservation_non_cancel_test() {
    let atomic_value = Arc::new(Mutex::new(0));
    let arc_clone = atomic_value.clone(); 
    let sender = time_task_reservation(move || {
        *arc_clone.lock().unwrap() = 1;
    }, Duration::from_millis(100));

    thread::sleep(Duration::from_millis(200));

    sender.send(Message::Stop);

    thread::sleep(Duration::from_millis(200));

    assert_eq!(1, *atomic_value.lock().unwrap());
}


