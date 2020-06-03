#![allow(unused)]

extern crate regex;

use std::net::{self,TcpStream};
use std::io::{Write,BufReader,BufWriter,BufRead,Error, Read};
use regex::Regex;

use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use std::ops::{Deref,DerefMut};

use super::server::{GameController,BufStream};
use super::game::{Player,PlayerOnMap};

impl GameController {
    pub fn announce_message(&mut self,msg : String) {
        let message_byte = &msg.into_bytes();
        for client_arc in &self.clients { 
            match client_arc.lock() {
                Ok(mut client) => {
                    client.write(message_byte);
                },
                Err(_) => {
                    println!("Could not send data");
                }
            }
        }
    }
    pub fn announce_message_byte(&mut self,msg : &[u8]) {
        for client_arc in &self.clients { 
            match client_arc.lock() {
                Ok(mut client) => {
                    client.write(msg);
                },
                Err(_) => {
                    println!("Could not send data");
                }
            }
        }
    }

    pub fn player_join_initialize(&mut self,mut stream : net::TcpStream) {
        let json = "{".to_string() + &format!(r#""counter":{}"#,(self.clients.len())) + "}|";
        match send_message(&stream,json) {
            Ok(_) => {
                println!("player joined! player details : {:?}",stream);
                self.player_join(stream);
            }
            Err(_) => {
                println!("error occured. this stream will ignore");
            }
        }
    }

    pub fn player_join(&mut self,mut stream : net::TcpStream) {
        //self.clients.lock().unwrap().push(stream);
        self.clients.push(Arc::new(Mutex::new(stream)));
        self.map.lock().unwrap().players.push(Player::new(0.0,0.0,0.0));
        self.map.lock().unwrap().players_on_map.push(PlayerOnMap::new(0,0,0));
    }
    
}

pub fn parse_client_info(msg : String) -> Vec<f32> {
    msg.split(',').map(|e| match(e.parse()) {
        Ok(o) => o,
        Err(_) => {
            //self.error(i);
            0.0
        }
    }).collect()
}

pub fn send_message(mut stream : &net::TcpStream,msg : String) -> Result<usize, Error> {
    stream.write(msg.as_bytes())
}
pub fn send_message_byte(mut stream : &net::TcpStream,msg : &[u8]) -> Result<usize, Error> {
    stream.write(msg)
}

pub fn read_by_buffer(br : Arc<Mutex<BufStream>>) -> String {
    let mut ret = String::new();
    let reader = &mut br.lock().expect("Could not lock buffer stream").rd;
    reader.read_line(&mut ret);
    ret
}

pub fn start_reading_coordinate(stream: &Vec<Arc<Mutex<BufStream>>>, data_buffer_non_static: &mut Vec<Arc<Mutex<String>>>) {
    // if data received, a mutex variable self.network_buffer change.
    for (i, stream_arc_non_static) in stream.iter().enumerate() {
        let stream_arc = stream_arc_non_static.clone(); // create 'static pointer
        let mut data_buffer = data_buffer_non_static.clone();
        thread::spawn(move || {
            loop {
                match(stream_arc.lock()) {
                    Ok(mut stream) => {
                        let mut read_data = String::new();
                        stream.rd.read_line(&mut read_data);
                        println!("got data from client {}", i);
                        *(data_buffer[i].lock().unwrap()) = read_data.clone();
                    },
                    Err(_) => {
                        println!("[warn] Could not lock client in start_reading_coordinate");
                    }
                }
            }
        });
    }
}
