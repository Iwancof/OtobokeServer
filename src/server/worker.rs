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


enum Order {
    Suspend,
    Restart,
    Destory,
}
enum Report {
    Success,
    CritError,
    GeneError,
}

//fn do_while_stop_with_recovery<>(task: impl<Fn() -> TResult>, recov: impl<Fn(Receiver<Report>) -> Report -> ()>) -> Self {

impl Worker {
    fn do_while_stop_with_recovery(
        task: Box<dyn Fn() -> Report + Send>, 
        recov: Box<dyn Fn(Sender<Report>) -> Box<dyn Fn(Report) -> ()>>,
        ) -> Self {
        let (order_sender, order_receiver) = channel(); // Out of thread -> Worker
        let (worker_report_sender, worker_report_receiver) = channel(); // Worker -> Out of thread
        let recovery = recov(worker_report_sender);

        let ret = Self {
            order_sender: order_sender,
            task_report: worker_report_receiver
        };

        thread::spawn(move || {
            // TODO name thread.
            let mut is_suspend = false;
            'main: loop {
                // main loop
                match order_receiver.try_recv() {
                    // if thread ordered.
                    Ok(order) => {
                        match order {
                            Order::Destory => {
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

                match task() {
                    // do task and catch error.
                    Report::Success => {
                        println!("Success");
                    },
                    Report::CritError => {
                        println!("Critical error occured.");
                        break 'main;
                    },
                    Report::GeneError => {
                        println!("General error occured. this will be ignore.");
                    },
                }
            }
        });
        ret
    }
}

