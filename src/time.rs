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
        mpsc::{
            channel,
            Sender,
        }
    },
};

// for test use
#[allow(unused)]
use std::ops::{Deref, DerefMut, Add, Sub};
use std::cmp::{PartialOrd};

use crate::server::worker::{
    WorkerTrait,
    Worker
};

const TIME_PER_COUNT: i128 = 10;

pub struct ClockWorker {
    workers: Vec<Worker>,
}



pub struct LoopTimerUnArc {
    latest: Instant,
    totals: Vec<u128>,
    tasks: Vec<Box<dyn Fn() -> () + Send>>,
    limits: Vec<u128>,
    count: usize,
}

impl LoopTimerUnArc {
    pub fn new() -> LoopTimerUnArc {
        LoopTimerUnArc {
            latest : Instant::now(),
            totals : vec![],
            tasks : vec![],
            limits : vec![],
            count : 0,
        }
    }

    pub fn subscribe(&mut self, func : Box<dyn Fn() -> () + Send>, limit : u128) -> usize {
       self.totals.push(0);
       self.tasks.push(func);
       self.limits.push(limit);
       self.count += 1;
       self.count
    }

    pub fn update(&mut self) {
        let dif = self.latest.elapsed().as_millis();
        self.latest = Instant::now();

        //Add diff
        for e in &mut self.totals {
            *e += dif;
        }
        //Check limit
        for i in 0..self.count {
            if self.limits[i] <= self.totals[i] {
                self.tasks[i]();
                self.totals[i] = 0;
            }
        }
    }
}

pub struct LoopTimer {
    item: Arc<Mutex<LoopTimerUnArc>>,
}

impl LoopTimer {
    pub fn new() -> LoopTimer {
        LoopTimer {
            item: Arc::new(Mutex::new(LoopTimerUnArc::new())) 
        }
    }
    pub fn subscribe(&mut self, func: Box<dyn Fn() -> () + Send>, limit: u128) -> usize {
        self.item.lock().unwrap().subscribe(func, limit)
    }
    pub fn start(&mut self) {
        let clone = self.item.clone();
        thread::spawn(move || {
            loop { 
                clone.lock().unwrap().update();
                thread::sleep(Duration::from_millis(50));
            }
        });
    }
}

#[test]
pub fn subscribe_test() {
    let mut timer = LoopTimer::new();
    let return_value = Arc::new(Mutex::new(0));
    let cloned = return_value.clone();
    timer.subscribe(Box::new(move || { *cloned.lock().unwrap().deref_mut() = 123; }) ,10 );
    timer.start();
    thread::sleep(Duration::from_millis(100));
    assert_eq!(*return_value.lock().unwrap().deref(),123);
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


