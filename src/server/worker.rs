#![allow(unused)]

use std::{
    sync::{
        Arc,
        Mutex,
        mpsc::{
            Sender,
        },
    },
};

/*
struct Worker {
    task: Box<dyn Fn() -> Result<(), Report> + Send>,
    sender: Sender<Order>,
    receiver: Receiver<Order>,
}
trait WorkerTrait {
    type OResult = Result<(), SendError<Order>>;
    type TResult = Result<(), Report>;

    fn order(&self, o: Order) -> Self::OResult;
    fn receive(&self) -> Option<Order>;
    fn do_task_once(&self) -> ::TResult;

    fn do_while_stop() {
        thread::spawn(move || {
            // TODO name thread.
            loop {
                // main loop
                match self.receive() {
                    // if thread ordered.
                    
                    Some(order) => {
                        match order {
                            Order::Stop => {
                                break;
                            }
                        }
                    },
                    None => ;,
                };

                match self.do_task_once() {
                    // do task and catch error.
                    Ok(_) => ;,
                    Err(err) => {
                        match err {
                            Report::Error => {
                                println!("Error occured");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    fn suspend(&self) -> OResult {
        self.order(Order::Suspend);
    }
    fn destory(self) -> OResult {
        self.order(Order::Destory)
    }
}
    

enum Order {
    Suspend,
    Destory,
}
enum Report {
    Error,
}



*/
