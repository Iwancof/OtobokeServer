#![allow(unused)]

extern crate regex;

use std::net::{self,TcpStream};
use std::io::{Write,BufReader,BufWriter,BufRead};
use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use std::time::Duration;
use regex::Regex;

use super::game::{Map,Player};

pub struct GameController {
    clients : Vec<TcpStream>,
    player_limit : usize,
    map : Map,
    error_counter : Vec<i32>,
    error_limit : i32,
}

impl GameController {
    pub fn new(map : Map) -> GameController {
        let l = 2;
        GameController{clients:Vec::new(),player_limit:l,map:map,error_counter:vec![0;l],error_limit:100}
    }
    
    pub fn error(&mut self,i : usize) {
        self.error_counter[i] += 1;
        if self.error_counter[i] >= self.error_limit {
            //panic!("[Error-panic]Client disconnected");
        }
    }

    pub fn wait_for_players(&mut self) {
        let listener = net::TcpListener::bind("localhost:8080").unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    //thread::spawn(move || {
                        self.player_join(stream)
                        //println!("{:?}",stream);
                    //});
                },
                Err(_) => {
                    println!("Unknown client detected.")
                }
            }
            if self.clients.len() >= self.player_limit {
                println!("Player limit reached.The game will start soon!");
                break;
            }
        }
    }
    fn player_join(&mut self,mut stream : net::TcpStream) {
        let json = "{".to_string() + &format!(r#""counter":{}"#,self.clients.len()) + "}|";
        match stream.write(json.as_bytes()) {
            Ok(_) => {
                println!("Player joined! Player details : {:?}",stream);
                self.clients.push(stream);
                self.map.players.push(Player::new(0.0,0.0,0.0));
            }
            Err(_) => {
                println!("Error occured. This stream will ignore");
            }
        }
    }
    pub fn start_game(&mut self) {
        self.distribute_map();

        let mut cant_get_data_count = vec![0,self.clients.len()];

        let buffer_streams : Arc<Mutex<Vec<BufStream>>>
            = Arc::new(Mutex::new(
            self.clients.iter().map(
            |c| BufStream::new(c,100)
        ).collect()
            ));
        let mut count = 0;

        loop { //main game loop 
            for i in 0..self.clients.len() {
                let cloned = buffer_streams.clone();
                let (sender,receiver) = mpsc::channel();
                
                thread::spawn(move || {
                    let mut locked = cloned.lock().unwrap();

                    let mut ret = String::new();
                    locked[i].rd.read_line(&mut ret).unwrap();
                    sender.send(ret);
                });

                match receiver.recv_timeout(Duration::from_millis(1000)) {
                    Ok(s) => {
                        if s.len() == 0 { //Disconnected or bat connection
                            self.error(i);
                        }
                        print!("\x1b[1;{}Hclient[{}]={}",i * 40 + 10,i,s);
                        let sp = s.split(',');
                        let ret : Vec<f32> = sp.map(|e| match(e.parse()) {
                            Ok(o) => o,
                            Err(_) => {
                                self.error(i);
                                0.0
                            }
                        }).collect();
                        //print_typename(ret);
                        if ret.len() != 3 {
                            self.error(i);
                        }
                        self.map.players[i].x = ret[0];
                        self.map.players[i].y = ret[1];
                        self.map.players[i].z = ret[2];
                        //println!("{:?}",ret);
                    }
                    Err(_) => { 
                        self.error(i);
                        continue; 
                    }
                }
            };
            //thread::sleep(Duration::from_millis(100));
            //println!("{}",self.map.coordinate_to_json());
            let received_data = format!("{}",self.map.coordinate_to_json());
            //println!("{}",received_data);
            //let bytes = &received_data[0..received_data.len()];
            for i in 0..self.clients.len() {
                self.clients[i].write(
                    received_data.as_bytes()
                 );
            }
            count += 1;
            //println!("{:?}",bytes);
        }
    }

    pub fn distribute_map(&mut self) {
        //let mut error_clients : Vec<&TcpStream> = vec![];
        let mut error_clients_index : Vec<usize> = vec![];
        let mut count = 0;
        for mut client in &self.clients {
            match client.write(format!("{}",self.map.map_to_string()).as_bytes()) {
                Ok(size) => {
                    println!("Write to {:?}, size = {}",&client,size);
                }
                Err(_) => {
                    println!("[Error]Could not send map data. The stream will exclude");
                    error_clients_index.push(count);
                }
            }
            let count_value = r#"{"value":""#.to_string() + &self.clients.len().to_string() + r#""}|"#;
            match client.write(count_value.as_bytes()) {
                Ok(size) => {
                    println!("Write to {:?}, size = {}",&client,size);
                }
                Err(_) => {
                    println!("[Error]Could not send map data. The stream will exclude");
                    error_clients_index.push(count);
                }
            }
            
            count += 1;
        }
        for e in &error_clients_index {
            self.clients.remove(*e);
        }
        println!("Map distributed");
    }
}

pub struct BufStream {
    rd : BufReader<TcpStream>,
    wr : BufWriter<TcpStream>,
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

fn print_typename<T>(_ : T) {
    //println!("type = {}",std::any::type_name::<T>());
}

