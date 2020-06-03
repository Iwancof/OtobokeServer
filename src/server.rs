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
use super::game;

pub struct GameController {
    //pub clients : Arc<Mutex<Vec<TcpStream>>>,
    pub clients : Vec<Arc<Mutex<TcpStream>>>,
    pub player_limit : usize,
    pub map : Arc<Mutex<Map>>,
    pub timer : LoopTimer, //This is doing tasks per the time
    pub network_buffer : Vec<Arc<Mutex<String>>>,
}

impl GameController {
    pub fn new(map : Map) -> GameController {
        let l = 1; //Players
        GameController{
            //clients : Arc::new(Mutex::new(Vec::new())),
            clients : vec![],
            player_limit : l,
            map : Arc::new(Mutex::new(map)),
            timer : LoopTimer::new(),
            network_buffer : vec![Arc::new(Mutex::new("0, 0, 0".to_string())); l]
        }
    }
    
    pub fn show_game_details(&self) {
        println!("Player limit : {}",self.player_limit);
    }

    pub fn wait_for_players(&mut self) {
        //let listener = net::TcpListener::bind("2400:4051:99c2:5800:9978:6c9:2c0c:8520:5522").unwrap(); //Create Listener
        let listener = net::TcpListener::bind("2400:4051:99c2:5800:11a4:53a7:248:a471:5522").unwrap(); //Create Listener
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

        let clone_clients_for_announce_pac_coordinate = self.clients.clone();
        let clone_clients_for_announce_bait_info = self.clients.clone();
        let clone_map_for_announce_pac_coordinate = self.map.clone();
        let clone_map_for_announce_bait_info = self.map.clone();
        self.timer.subscribe(Box::new(move || { //Per 0.2 seconds, Program executes this closure.
            clone_map_for_announce_pac_coordinate.lock().unwrap().move_pacman();
            let msg = &clone_map_for_announce_pac_coordinate.lock().unwrap().coordinate_to_json_pacman().into_bytes();
                //Pacman coordinate convert to json here.
            for client_arc in &clone_clients_for_announce_pac_coordinate {
                match client_arc.lock() {
                    Ok(mut client) => {
                        client.write(msg); //And send to clients. (Can't use announce methot)
                    },
                    Err(_) => {
                        println!("Could not send pacman coordiante for client");
                    }
                }
            }
        }),
        200 //Time span
        );
        self.timer.subscribe(Box::new(move || {
            let msg = game::paced_vec_to_string(clone_map_for_announce_bait_info.lock().unwrap().paced_collection.lock().unwrap().clone()).into_bytes();
            
            for client_arc in &clone_clients_for_announce_bait_info {
                match client_arc.lock() {
                    Ok(mut client) => {
                        client.write(&msg); //And send to clients. (Can't use announce methot)
                    },
                    Err(_) => {
                        println!("Could not send pacman coordiante for client");
                    }
                }
            }
            *clone_map_for_announce_bait_info.lock().unwrap().paced_collection.lock().unwrap() = vec![];
        }),200);

        self.distribute_map();

        let buffer_streams : Vec<Arc<Mutex<BufStream>>> // USAGE: access by index and mutex proc
            = self.clients.iter().map(|c| // each clients
                match(c.lock()) {
                    Ok(client) => {
                        Arc::new(
                            Mutex::new(
                                BufStream::new(&client) // make stream from clients
                                )
                            )
                    },
                    Err(_) => {
                        panic!("Error occured in initialize network");
                    }
                }
                ).collect();
       
        
        let client_count = self.player_limit;

        // wait client effect
        let mut receivers = vec![];
        for i in 0..client_count {
            let cloned : Arc<Mutex<BufStream>> = buffer_streams[i].clone();
            let (sender, receiver) = mpsc::channel();
            receivers.push(receiver);
            thread::spawn(move || {
                sender.send(network::read_by_buffer(cloned));
            });
        }

        for r in receivers {
            match(r.recv()) {
                Ok(msg) => {
                    if msg != "END_EFFECT\n" {
                        panic!("Invalid message in wait client effect. data is {}", msg);
                    }
                },
                Err(_) => {
                    panic!("Receiver got error in waiting client effect.");
                }
            }
        }

        println!("GameReady!!");
        self.announce_message("GameReady|".to_string());


        self.timer.start(); //Start doing tasks

        network::start_reading_coordinate(&buffer_streams, &mut self.network_buffer.clone());

        let mut debug_count = 0;
        loop { //main game loop 
            for i in 0..client_count {
                let player_lastest_message = self.network_buffer[i].lock().unwrap().clone();
                // get lastest player info
                debug_count += 1;
                debug_count %= 100;
                if debug_count == 0 {
                    println!("\x1b[10;0Hclient[{}] at {}",i,&player_lastest_message);
                    //println!("client[{}] at {}", i, player_lastest_message.clone);
                }
                let parse_result = network::parse_client_info(player_lastest_message);
                self.map.lock().unwrap().update_coordinate(i, parse_result);
            };
            let received_data = self.map.lock().unwrap().coordinate_to_json();
            self.announce_message(received_data); // announce clients coordinate to clients
        }
    }

}

pub struct BufStream {
    pub rd : BufReader<TcpStream>,
    pub wr : BufWriter<TcpStream>,
}

impl BufStream {
    pub fn new(stream : &TcpStream) -> BufStream {
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

