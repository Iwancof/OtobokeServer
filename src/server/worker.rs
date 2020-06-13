#![allow(unused)]

use std::{
    sync::{
        Arc,
        Mutex,
        mpsc::{
            Sender,
            SendError,
            Receiver,
        },
    },
    thread,
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
    fn do_task_once(&self) -> Self::TResult;

    fn do_while_stop(&self) {
        thread::spawn(move || {
            // TODO name thread.
            let mut is_suspend = false;
            'main: loop {
                // main loop
                match self.receive() {
                    // if thread ordered.
                    
                    Some(order) => {
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
                    None => { },
                };

                if is_suspend {
                    continue 'main;
                }

                match self.do_task_once() {
                    // do task and catch error.
                    Ok(_) => { },
                    Err(err) => {
                        match err {
                            Report::CritError => {
                                println!("Critical error occured.");
                                break 'main;
                            },
                            Report::GeneError => {
                                println!("General error occured. this will be ignore.");
                            },
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
    Restart,
    Destory,
}
enum Report {
    /// true: 処理を続ける
    /// false: 処理を中断する
    CritError,
    GeneError,
}


*/
