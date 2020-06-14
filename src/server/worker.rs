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

pub struct Worker {
    order_sender: Sender<Order>,
    task_report: Receiver<Report>,
}
type OResult = Result<(), SendError<Order>>;

pub trait WorkerTrait
    where
        Self: Sized,
{
    // require methot.
    fn order(&self, o: Order) -> OResult; // Out of thread -> Worker (Order)
    fn receive(&self) -> Option<Report>;  // Worker -> Out of thread (Report)
    fn wait_receive(&self) -> Report;

    // provide methot.
    fn once(&self) -> OResult {
        self.order(Order::Once)
    }
    fn restart(&self) -> OResult {
        self.order(Order::Restart)
    }
    fn suspend(&self) -> OResult {
        self.order(Order::Suspend)
    }
    fn destory(&self) -> OResult {
        self.order(Order::Destory)
    }
}


#[derive(Debug, PartialEq)]
pub enum Order {
    Once,
    Suspend,
    Restart,
    Destory,
}

#[derive(Debug, PartialEq)]
pub enum Report {
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
    fn wait_receive(&self) -> Report {
        match self.task_report.recv() {
            Ok(rep) => rep,
            Err(_) => panic!("task not responce"),
        }
    }
}


impl Worker {
    pub fn do_while_stop_with_recovery
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
                            Order::Once => {
                                is_suspend = true;
                                recovery(task());
                                continue 'main
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
    // Recommend
    /// 推薦
    pub fn do_while_stop_report_error
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
}

