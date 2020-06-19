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
        mpsc::{
            sync_channel,
        },
    },
    thread,
    time::{
        Duration,
    },
};

use super::{
    GameController,
};

use crate::{
    game,
    map::MapInfo,
};



#[test]
fn server_tester() {
    println!("\n\n--------------------\nconnection test.\n\n");
    let mut test_data = Arc::new(Mutex::new(vec![]));
    let map_string = Arc::new("01010\n00000\n31014\n10110\n01110".to_string());
    {
        let test_data = test_data.clone();
        let map_string = map_string.clone();
        let (s, _) = sync_channel(0);
        //let mut g = GameController::new(game::Game::new(MapInfo::build_by_string(map_string.to_string()), 1), s);
        let game_instance = Arc::new(Mutex::new(
                game::Game::new(MapInfo::build_by_string(map_string.to_string()), 1), s));
        let instance_clone = game_instance.clone();
        
        let inner_test = thread::spawn(move || {
            instance_clone.lock().unwrap().server_flow_tmp();
            thread::sleep(Duration::from_millis(2000));
            panic!("server test timeout");
        });

        match TcpStream::connect("192.168.1.9:5522") {
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
        game_instance.lock().unwrap().stop();
    }
    assert_eq!(String::from_utf8(test_data.lock().unwrap().to_vec()).unwrap(), map_string.to_string().replace("\n", ",") + ",|");
    println!("--------------------\n\n\n");
}

