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

pub struct LazyWorker {
    counter: u128,
    threshold: u128,
    worker: Arc<Worker>,
}
pub struct WorkersConductor {
    // FIXME too big overhead. bad precise
    latest: Mutex<Option<Instant>>,
    workers: Mutex<Vec<LazyWorker>>,
}
pub enum ConductorOrder {
    Stop,
}
impl LazyWorker {
    pub fn new_with_worker(thres: u128, wok: Worker) -> Self {
        Self {
            counter: 0,
            threshold: thres,
            worker: Arc::new(wok),
        }
    }
    pub fn new(thres: u128, task: Box<dyn Fn() -> WorkerResult + Send + Sync>, sender: SyncSender<String>) -> Self {
        Self::new_with_worker(thres, Worker::new(task, sender))
    }
    pub fn update(&mut self, count: u128) {
        self.counter += count;
        if self.counter > self.threshold {
            let task_ptr = self.worker.clone();
            thread::Builder::new().name("Lazy task".to_string()).spawn(move || {
                task_ptr.run();
            });
            self.counter -= self.threshold;
        }
    }
}
impl WorkersConductor {
    fn new() -> Self {
        Self {
            latest: Mutex::new(None),
            workers: Mutex::new(vec![]),
        }
    }
    fn update(self: Arc<Self>) {
        let diff = self.update_and_get_diff();
        for worker in &mut *self.workers.lock().unwrap() {
            worker.update(diff);
        }
    }
    fn push(self: &Arc<Self>, worker: LazyWorker) {
        self.workers.lock().unwrap().push(worker)
    }

    pub fn start(self: &Arc<Self>) -> Sender<ConductorOrder> {
        let mut timer_ptr = self.latest.lock().unwrap();
        if let Some(_) = *timer_ptr {
            panic!("timer haves already started");
        };
        let (sender, receiver) = channel();
        let self_clone = self.clone();
        *timer_ptr = Some(Instant::now());

        thread::Builder::new()
            .name("Worker conductor thread".to_string()).spawn(move || {
                loop {
                    match receiver.try_recv() {
                        Ok(ConductorOrder::Stop) => {
                            println!("Timer stop!");
                            break;
                        },
                        _ => {},
                    };
                    self_clone.clone().update();
                    thread::sleep(Duration::from_millis(20));
                    
                }
            });
        sender
    }
    fn update_and_get_diff(self: &Arc<Self>) -> u128 {
        let ret = self.latest.lock().unwrap().unwrap().elapsed().as_millis();
        let ret = match *self.latest.lock().unwrap() {
            Some(t) => t.elapsed().as_millis(),
            None => { panic!("timer is not started"); },
        };
        *self.latest.lock().unwrap() = Some(Instant::now());
        ret
    }
}

#[test]
fn timer_test() {
    let workers_conductor = Arc::new(WorkersConductor::new());
    let (sender, receiver) = sync_channel(16);
    
    let task1_counter = Arc::new(Mutex::new(0));
    let task2_counter = Arc::new(Mutex::new(0));

    let ltask1 = {
        let counter = task1_counter.clone();
        LazyWorker::new(
            100,
            Box::new(move || {
                println!("TASK1");
                *counter.lock().unwrap() += 1;
                WorkerResult::Complete
            }),
            sender.clone()
        )
    };
    let ltask2 = {
        let counter = task2_counter.clone();
        LazyWorker::new(
            100,
            Box::new(move || {
                println!("TASK2");
                *counter.lock().unwrap() += 1;
                WorkerResult::Complete
            }),
            sender.clone()
        )
    };

    workers_conductor.push(ltask1);
    workers_conductor.push(ltask2);

    let conductor_sender = workers_conductor.start();
    thread::sleep(Duration::from_millis(550));

    assert_eq!(*task1_counter.lock().unwrap(), *task2_counter.lock().unwrap());
    assert_eq!(5, *task2_counter.lock().unwrap());
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


