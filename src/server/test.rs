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
    let map_string = Arc::new("01010\n00000\n31014\n10110\n01110".to_string());

    let (s, _) = sync_channel(0);
    let game_instance = Arc::new(Mutex::new(GameController::new(
            game::Game::new(MapInfo::build_by_string(map_string.to_string()), 1), s)));
    let instance_clone = game_instance.clone();
    
    let inner_test = thread::spawn(move || {
        instance_clone.lock().unwrap().server_flow_to_test();
        thread::sleep(Duration::from_millis(2000));
        panic!("server test timeout");
    });

    match TcpStream::connect("127.0.0.1:5522") {
        Ok(stream) => {
            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            let mut data = vec![];
            reader.read_until(b'|', &mut data).expect("Server does not responce");
            assert_eq!(String::from_utf8(data).unwrap(), "{\"counter\":0}|");

            let mut data = vec![];
            reader.read_until(b'|', &mut data).expect("Server does not responce");
            assert_eq!(String::from_utf8(data).unwrap(), "StartGame|");

            let mut data = vec![];
            reader.read_until(b'|', &mut data).expect("Server does not responce");
            assert_eq!(String::from_utf8(data).unwrap(), 
                       map_string.to_string().replace("\n", ",") + ",|");

            let mut data = vec![];
            reader.read_until(b'|', &mut data).expect("Server does not responce");
            assert_eq!(String::from_utf8(data).unwrap(), "{\"value\":\"1\"}|");

            writer.write("END_EFFECT\n".as_bytes()).expect("write failed.");
            writer.flush().unwrap();

            let mut data = vec![];
            reader.read_until(b'|', &mut data).expect("Server does not responce");
            assert_eq!(String::from_utf8(data).unwrap(), "THIS IS TEST MESSAGE|");
        }
        Err(_) => {
            panic!("Connection NG.");
        }
    }
}

