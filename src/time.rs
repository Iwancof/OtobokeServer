#![deny(unused)]
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
    },
};

// for test use
#[allow(unused)]
use std::ops::{Deref, DerefMut};

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
