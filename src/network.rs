#![allow(unused)]

extern crate regex;

use std::net::{self,TcpStream};
use std::io::{Write,BufReader,BufWriter,BufRead,Error};
use regex::Regex;

use std::sync::{mpsc,Arc,Mutex};
use std::thread;
use std::ops::{Deref,DerefMut};

use super::server::{GameController,BufStream};
use super::game::{Player,PlayerOnMap};

impl GameController {
    pub fn announce_message(&mut self,msg : String) {
        let message_byte = &msg.into_bytes();
        for client in &mut *self.clients.lock().unwrap().deref_mut() { 
            client.write(
                message_byte
             );
        }
    }
    pub fn announce_message_byte(&mut self,msg : &[u8]) {
        for client in &mut *self.clients.lock().unwrap().deref_mut() { 
            client.write(
                msg
             );
        }
    }

    pub fn player_join_initialize(&mut self,mut stream : net::TcpStream) {
        let json = "{".to_string() + &format!(r#""counter":{}"#,(*self.clients.lock().unwrap().deref()).len()) + "}|";
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
        self.clients.lock().unwrap().push(stream);
        self.map.lock().unwrap().players.push(Player::new(0.0,0.0,0.0));
        self.map.lock().unwrap().players_on_map.push(PlayerOnMap::new(0,0,0));
    }
    
    /*
    pub fn deref(&mut self) -> Result<&Vec<TcpStream>,String> {
        match self.clients.lock() {
            Ok(x) => Ok(x.deref()),
            Err(_) => Err("Could not lock clients".to_string()),
        }
    }
    */
}

pub fn enable_sender(clone_stream : Arc<Mutex<Vec<BufStream>>>,sender : mpsc::Sender<String>,i : usize) {
    thread::spawn(move || {
        let mut locked = clone_stream.lock().unwrap(); //TODO: use match

        let mut ret = String::new();

        locked[i].rd.read_line(&mut ret).unwrap();
        sender.send(ret);
    });
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

