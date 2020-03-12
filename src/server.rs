#![allow(unused)]

extern crate regex;

use std::net::{self,TcpStream};
use std::io::{Write,BufReader,BufWriter,BufRead};
use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use std::time::Duration;
use regex::Regex;

use super::game::{Map,Player};
use super::network;
use super::time::{LoopTimer};

pub struct GameController {
    pub clients : Vec<TcpStream>,
    pub player_limit : usize,
    pub map : Arc<Mutex<Map>>,
    pub timer : LoopTimer, //This is doing tasks per the time
}

impl GameController {
    pub fn new(map : Map) -> GameController {
        let l = 2; //Players
        GameController{
            clients : Vec::new(),
            player_limit : l,
            map : Arc::new(Mutex::new(map)),
            timer : LoopTimer::new(),
        }
    }
    
    pub fn wait_for_players(&mut self) {
        let listener = net::TcpListener::bind("localhost:8080").unwrap(); //Create Listener

        for stream in listener.incoming() { //Wait for players
            match stream {
                Ok(stream) => { self.player_join_initialize(stream) },
                Err(_) => { println!("Unknown client detected.") }
            }

            if self.clients.len() >= self.player_limit {
                println!("Player limit reached. The game will start soon!");
                return;
            }
        }
    }

    pub fn distribute_map(&mut self) {
        let map_data = self.map.lock().unwrap().map_to_string();
        self.announce_message(map_data);
    }
   
    pub fn start_game(&mut self) {
        self.announce_message("StartGame|".to_string());


        let clone = self.map.clone();
        self.timer.subscribe(Box::new(move || {
            //clone.lock().unwrap().print_onmap_coordinate();
            //clone.lock().unwrap().print_map(0,0);
        }),100);
        self.timer.start(); //Start doing tasks

        self.distribute_map();

        //let mut cant_get_data_count = vec![0,self.clients.len()];

        let buffer_streams : Arc<Mutex<Vec<BufStream>>>
            = Arc::new(Mutex::new(
            self.clients.iter().map(
            |c| BufStream::new(c,100)
        ).collect()
            ));
        //let mut count = 0;

        loop { //main game loop 
            //self.map.pacman.x += 1.; //for test
            for i in 0..self.clients.len() {
                let cloned = buffer_streams.clone();
                let (sender,receiver) = mpsc::channel(); //Sender<String>,Receiver<String>

                network::enable_sender(cloned,sender,i); //Listening stream in new thread

                match receiver.recv_timeout(Duration::from_millis(1000)) {
                    Ok(s) => {
                        //print!("\x1b[1;{}Hclient[{}]={}",i * 40 + 10,i,s);
                        let ret = network::parse_client_info(s);
                        self.map.lock().unwrap().players[i].x = ret[0];
                        self.map.lock().unwrap().players[i].y = ret[1];
                        self.map.lock().unwrap().players[i].z = ret[2];
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

