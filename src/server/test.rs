#![allow(unused)]
#![allow(dead_code)]

use std::{
    net::{
        self,
        TcpStream,
    },
    io::{
        Write,
        BufRead,
        BufReader,
        BufWriter,
    },
    sync::{
        Arc,
        Mutex,
    },
    thread,
};

use super::{
    GameController,
};

use crate::{
    game,
    map::MapInfo,
};



//#[test]
// wait for implement time spaned worker
fn server_tester() {
    println!("\n\n--------------------\nconnection test.\n\n");
    let mut test_data = Arc::new(Mutex::new(vec![]));
    let map_string = Arc::new("01010\n00000\n31014\n10110\n01110".to_string());
    {
        let test_data = test_data.clone();
        let map_string = map_string.clone();
        let server_test = thread::spawn(move || {
            let inner_test = thread::spawn(move || {
                let mut g = GameController::new(game::Game::new(MapInfo::build_by_string(map_string.to_string()), 1));
                g.server_flow_tmp();
            });

            match TcpStream::connect("localhost:5522") {
                Ok(stream) => {
                    println!("connection ok.");
                    let mut ret = String::new();
                    let mut reader = BufReader::new(&stream);
                    let mut writer = BufWriter::new(&stream);
                    writer.write("END_EFFECT\n".as_bytes()).expect("write failed.");
                    writer.flush().unwrap();
                    for i in 0.. 2 {
                        let mut data = vec![];
                        match reader.read_until(b'|', &mut data) {
                            Ok(_) => println!("READ: {:?}", String::from_utf8(data).unwrap()),
                            Err(_) => println!("could not read."),
                        }
                    }
                    println!("aaaaaa");
                    let result = reader.read_until(b'|', &mut test_data.lock().unwrap());
                    match result {
                        Ok(_) => println!("READ: {:?}", String::from_utf8(test_data.lock().unwrap().to_vec()).unwrap()),
                        Err(_) => println!("could not read."),
                    }
                    for i in 0.. 2 {
                        let mut data = vec![];
                        match reader.read_until(b'|', &mut data) {
                            Ok(_) => println!("READ: {:?}", String::from_utf8(data).unwrap()),
                            Err(_) => println!("could not read."),
                        }
                    }
                }
                Err(_) => {
                    panic!("Connection NG.");
                }
            }
            return;
        });
        server_test.join().unwrap();
    }
    assert_eq!(String::from_utf8(test_data.lock().unwrap().to_vec()).unwrap(), map_string.to_string().replace("\n", ",") + ",|");
    println!("--------------------\n\n\n");
}

#[test]
fn worker_test() {
    use super::worker::{
        WorkerTrait,
        Worker,
        Order,
        Report,
    };
    use std::time::Duration;
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
                    3 => Report::Timeout,
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
                            Report::Timeout => { sender.send(Report::Timeout); },
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
                    3 => Report::Timeout,
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

#[test]
fn worker_once_test() {
    use super::worker::{
        WorkerTrait,
        Worker,
        Order,
        Report,
    };
    use std::time::Duration;

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let worker = Worker::do_while_stop_with_recovery(
        "thread for worker_once_test",
        Box::new(move || {
            *counter_clone.lock().unwrap() += 1;
            Report::Success
        }),
        Box::new(move | sender | {
            Box::new(move | report | {
                sender.send(report);
            })
        }),
    );
    worker.suspend();
    assert_eq!(*counter.lock().unwrap(), 0);
    worker.once(); // 1
    worker.wait_receive();
    assert_eq!(*counter.lock().unwrap(), 1);
    worker.once(); // 2
    worker.once(); // 3
    worker.once(); // 4
    worker.wait_receive();
    assert_eq!(*counter.lock().unwrap(), 4);
}
