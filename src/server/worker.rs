#![allow(unused)]

use std::{
    sync::{
        Arc,
        Mutex,
        mpsc::{
            Sender,
            SendError,
            Receiver,
            channel,
        },
    },
    clone::{
        Clone,
    },
    time::{
        Duration,
    },
    thread,
};

struct Worker {
    order_sender: Sender<Order>,
    task_report: Receiver<Report>,
}
type OResult = Result<(), SendError<Order>>;

trait WorkerTrait
    where
        Self: Sized,
{
    // require methot.
    fn order(&self, o: Order) -> OResult; // Out of thread -> Worker (Order)
    fn receive(&self) -> Option<Report>;  // Worker -> Out of thread (Report)

    // provide methot.
    fn suspend(&self) -> OResult {
        self.order(Order::Suspend)
    }
    fn destory(&self) -> OResult {
        self.order(Order::Destory)
    }
}


#[derive(Debug, PartialEq)]
enum Order {
    Suspend,
    Restart,
    Destory,
}

#[derive(Debug, PartialEq)]
enum Report {
    Success,
    CritError,
    GeneError,
}

impl WorkerTrait for Worker {
    fn order(&self, o:Order) -> OResult {
        self.order_sender.send(o)
    }
    fn receive(&self) -> Option<Report> {
        match self.task_report.try_recv() {
            Ok(rep) => Some(rep),
            Err(_) => None,
        }
    }
}


impl Worker {
    fn do_while_stop_with_recovery
        (
            task_name: &str,
            // task_name
            task: Box<dyn Fn() -> Report + Send>, 
            // worker's task
            recov: Box<dyn Fn(Sender<Report>) -> Box<dyn Fn(Report) -> () + Send>>,
            // if task report some error, recov(sender) calls.
        ) -> Self {
        let (order_sender, order_receiver) = channel(); // Out of thread -> Worker
        let (worker_report_sender, worker_report_receiver) = channel(); // Worker -> Out of thread
        let recovery = recov(worker_report_sender);

        let ret = Self {
            order_sender: order_sender,
            task_report: worker_report_receiver
        };

        let name = task_name.to_string();

        thread::Builder::new().name(name.to_string()).spawn(move || {
            // TODO name thread.
            let mut is_suspend = false;
            'main: loop {
                // main loop
                match order_receiver.try_recv() {
                    // if thread ordered.
                    Ok(order) => {
                        match order {
                            Order::Destory => {
                                println!("Destory thread. name: {}", name);
                                break 'main;
                            }, 
                            Order::Suspend => {
                                is_suspend = true;
                            },
                            Order::Restart => {
                                is_suspend = false;
                            },
                        }
                    },
                    Err(_) => { },
                };

                if is_suspend {
                    continue 'main;
                }

                recovery(task());
            }
        });
        ret
    }
    fn do_while_stop_report_error
        (
            task_name: &str,
            task: Box<dyn Fn() -> Report + Send>,
        ) -> Self {
            Self::do_while_stop_with_recovery(
                task_name,
                task,
                Box::new(move | sender | {
                    Box::new(move | report | {
                        if report != Report::Success {
                            sender.send(report);
                        }
                    })
                }),
            )
    }
    // Recommend
    /// 推薦
    fn do_task(task: Box<dyn Fn() -> Report + Send>) -> Self {
        Self::do_while_stop_report_error("unnamed task", task)
    }
    fn run(task: Box<dyn Fn() -> () + Send>) -> Self {
        let task_report = Box::new(move || {
            task();
            Report::Success
        });
        Self::do_task(task_report)
    }
}

#[test]
fn worker_test() {
    let thread_counter = Arc::new(Mutex::new(0));
    let worker = Worker::do_while_stop_with_recovery(
        "thread for worker_test",
        Box::new(
            move || {
                // do something
                thread::sleep(Duration::from_millis(10));
                let ret = match *thread_counter.lock().unwrap() {
                    0 => Report::Success,
                    1 => Report::GeneError,
                    2 => Report::CritError,
                    _ => Report::GeneError,
                };
                *thread_counter.lock().unwrap() += 1;
                ret
            },
        ),
        Box::new(
            move | sender | {
                Box::new(
                    move | report | {
                        match report {
                            Report::Success => { sender.send(Report::Success); },
                            Report::GeneError => { sender.send(Report::GeneError); },
                            Report::CritError => { sender.send(Report::CritError); }
                        }
                    },
                )
            },
        ),
    );
    let mut counter = 0;
    loop {
        match worker.receive() {
            Some(rep) => {
                let left = rep;
                let right = match counter {
                    0 => Report::Success,
                    1 => Report::GeneError,
                    2 => Report::CritError,
                    _ => { break; },
                };
                assert_eq!(left, right);
                counter += 1;
            },
            None => {
                /* */
            },
        }
    }
}
