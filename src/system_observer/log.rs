#[allow(unused)]

use std::sync::{
    mpsc::{
        Sender,
        Receiver,
        SyncSender,
        sync_channel,
    },
    Arc,
    Mutex,
};
use std::{
    thread,
};

use crate::lazy_static;

lazy_static! {
    static ref RECEIVERS:
        Arc<Mutex<Vec<(usize, Receiver<String>)>>> =
        Arc::new(Mutex::new(vec![]));
        // usize is instance id, Receiver is instance's thread error.
}
lazy_static! {
    static ref SERVER_COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

fn issue_messenger() -> SyncSender<String> {
    let (sender, receiver) = sync_channel(16);
    *(*SERVER_COUNTER).lock().unwrap() += 1;
    (*RECEIVERS).lock().unwrap().push((*(*SERVER_COUNTER).lock().unwrap(), receiver));

    sender 
}

fn start_try_recv_error() {
    thread::Builder::new().name("error receiver".to_string()).spawn(move || {
        for (id, rec) in &*RECEIVERS.lock().unwrap() {
            match rec.try_recv() {
                Ok(msg) => {
                    println!("Pollution detect!!!! in instance {}. message:{}", id, msg);
                },
                Err(_) => { // No problems!

                }
                
            }
            
        }
    });

}

