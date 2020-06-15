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
    collections::{
        HashSet,
        LinkedList,
    }
};

// for test use
#[allow(unused)]
use std::ops::{Deref, DerefMut, Add, Sub};
use std::cmp::{PartialOrd};

use crate::server::worker::{
    WorkerTrait,
    Worker,
    Report,
};

pub struct ClockWorker {
    pub workers: Arc<Mutex<Vec<(u128, u128, Worker<Report>)>>>,
    // task has (counter, frecquency, Worker)
    instructor: Worker<LinkedList<(usize, Report)>>,
    // if tasks occure error, instructor report set of (task_id, task_error).
}

impl ClockWorker {
    fn new() -> Self {
        let timer = Arc::new(Mutex::new(Instant::now()));
        let workers = Arc::new(Mutex::new(vec![]));
        let workers_clone = workers.clone();
        Self {
            workers: workers,
            instructor: Worker::<LinkedList::<(usize, Report)>>::do_while_stop_with_recovery(
                "ClockWorker's timer",
                Box::new(move || {
                    let count = timer.lock().unwrap().elapsed().as_millis();
                    for work in workers_clone.lock().unwrap().iter_mut() {
                        work.0 += count;
                        if work.0 > work.1 {
                            work.0 -= work.1;
                            work.2.once().unwrap();
                        }
                    }
                    let mut ret = LinkedList::new();
                    // wait all tasks result.
                    
                    for (i, work) in workers_clone.lock().unwrap().iter_mut().enumerate() {
                        match work.2.timeout_receive() {
                            Some(report) => {
                                ret.push_back((i, report));
                            },
                            None => {
                                ret.push_back((i, Report::Timeout));
                            }
                        }
                    }

                    ret
                }),
                Box::new( move | sender | {
                    Box::new( move | reports | {
                        for report in reports.iter() {
                            match report {
                                (index, Report::Success) => {
                                    println!("in thread{} Success thread", index);
                                },
                                (index, Report::Timeout) => {
                                    println!("in thread{} General error occures", index);
                                },
                                (index, Report::CritError) => {
                                    println!("in thread{} Critical error occures", index);
                                },
                                (index, Report::GeneError) => {
                                    println!("in thread{} Timeout", index);
                                },
                            }
                        }
                    })
                }),
            ),
        }
    }
}

#[test]
fn clock_worker_test() {
    let mut clock_worker = ClockWorker::new();
    let counter = Arc::new(Mutex::new(0));
    clock_worker.workers.lock().unwrap().push(
        (100, 0, Worker::do_while_stop_with_recovery(
                "thread for clock_worker_test",
                Box::new(move || {
                    let ptr: &mut i32  = &mut *counter.lock().unwrap();
                    *ptr += 1;
                    thread::sleep(Duration::from_millis(10));
                    match *ptr % 4 {
                        0 => Report::Success,
                        1 => Report::CritError,
                        2 => Report::GeneError,
                        3 => Report::Success,
                        _ => {
                            panic!("Error occures");
                        }
                    }
                }),
                Box::new(move | sender | {
                    Box::new(move | report | {
                        sender.send(report);
                    })
                }),
        )));
    thread::sleep(Duration::from_millis(1000));

    panic!();
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


