#![allow(unused)]

extern crate regex;

use std::net::{self,TcpStream};
use std::io::{Write,BufReader,BufWriter,BufRead};
use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use std::time::Duration;
use regex::Regex;

use std::ops::{Deref,DerefMut};

use super::game::{Map,Player};
use super::network;
use super::time::{LoopTimer};

pub struct GameController {
    pub clients : Arc<Mutex<Vec<TcpStream>>>,
    pub player_limit : usize,
    pub map : Arc<Mutex<Map>>,
    pub timer : LoopTimer, //This is doing tasks per the time
}

impl GameController {
    pub fn new(map : Map) -> GameController {
        let l = 1; //Players
        GameController{
            clients : Arc::new(Mutex::new(Vec::new())),
            player_limit : l,
            map : Arc::new(Mutex::new(map)),
            timer : LoopTimer::new(),
        }
    }
    
    pub fn show_game_details(&self) {
        println!("Player limit : {}",self.player_limit);
    }

    pub fn wait_for_players(&mut self) {
        let listener = net::TcpListener::bind("localhost:8080").unwrap(); //Create Listener
        let mut count = 0;

        for stream in listener.incoming() { //Wait for players
            match stream {
                Ok(stream) => { self.player_join_initialize(stream);count += 1; },
                Err(_) => { println!("Unknown client detected.") }
            }

            if count >= self.player_limit {
                println!("Player limit reached. The game will start soon!");
                return;
            }
        }
    }

    pub fn distribute_map(&mut self) {
        let map_data = self.map.lock().unwrap().map_to_string();
        self.announce_message(map_data);
        self.announce_message(
            r#"{"value":""#.to_string() + &(self.player_limit.to_string()) + r#""}|"#);
        println!("map disributed");
    }
   
    pub fn start_game(&mut self) {
        self.announce_message("StartGame|".to_string());

        let clone_client = self.clients.clone();
        let clone_map = self.map.clone();
        self.timer.subscribe(Box::new(move || { //Per 0.2 seconds, Program executes this closure.
            clone_map.lock().unwrap().move_pacman();
            let msg = &clone_map.lock().unwrap().coordinate_to_json_pacman().into_bytes();
                //Pacman coordinate convert to json here.
            for client in &mut *clone_client.lock().unwrap().deref_mut() {
                client.write(msg); //And send to clients. (Can't use announce methot)
            }
            //clone_map.lock().unwrap().debug_next();
        }),
        200 //Time span
        );
        self.timer.start(); //Start doing tasks

        self.distribute_map();

        let buffer_streams : Arc<Mutex<Vec<BufStream>>>
            = Arc::new(Mutex::new(
            self.clients.lock().unwrap().deref_mut().iter().map(
            |c| BufStream::new(c,100)
        ).collect()
            ));
        //let mut count = 0;

        loop { //main game loop 
            for i in 0..self.clients.lock().unwrap().deref().len() {
                let cloned = buffer_streams.clone();
                let (sender,receiver) = mpsc::channel(); //Sender<String>,Receiver<String>

                network::enable_sender(cloned,sender,i); //Listening stream in new thread

                match receiver.recv_timeout(Duration::from_millis(1000)) {
                    Ok(s) => {
                        let ret = network::parse_client_info(s);
                        self.map.lock().unwrap().update_coordinate(i,ret);
                    }
                    Err(_) => { 
                        continue; //Could not receive data
                    }
                }
            };
            let received_data = self.map.lock().unwrap().coordinate_to_json();
            self.announce_message(received_data); //Announce clients coordinate to clients
        }
    }

}

pub struct BufStream {
    pub rd : BufReader<TcpStream>,
    pub wr : BufWriter<TcpStream>,
}

impl BufStream {
    pub fn new(stream : &TcpStream,limit : i32) -> BufStream {
        BufStream{
            rd : BufReader::new(
                stream.try_clone().unwrap()),
            wr : BufWriter::new(
                stream.try_clone().unwrap())}
    }
}

pub fn print_typename<T>(_ : T) {
    println!("type = {}",std::any::type_name::<T>());
}

