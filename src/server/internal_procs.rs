#![allow(unused)]
#![allow(dead_code)]

use std::{
    net::{
        self,
        TcpStream,
    },
    io::{
        Write,
        BufReader,
        BufWriter,
    },
    sync::{
        mpsc,
        Arc,
        Mutex,
        mpsc::{
            SyncSender,
        },
    },
    thread,
};

use crate::{
    game::Game,
    game,
    map::{
        self,
        GameClient,
    },
    time::{
        WorkerConductor,
    },
};

use super::{
    GameController,
    communication::{
        CommunicationProviderTrait,
        CommunicationProvider,
        read_by_buffer,
    },
};

impl GameController {
    pub fn new(game: Game, snd: SyncSender<String>) -> GameController {
        GameController{
            player_limit: game.number_of_player,
            game: Arc::new(Mutex::new(game)),
            comn_prov: Arc::new(Mutex::new(CommunicationProvider::new())),
            conduc: WorkerConductor::new(snd.clone()),
            log_sender: snd,
        }
    }
    
    pub(super) fn wait_until_clients_connection(&mut self, address: &str) {
        let listener = net::TcpListener::bind(address).unwrap(); //Create Listener
        let mut count = 0;

        if !cfg!(test) {
            self.log_sender.send(format!("Server started listening at {}", address));
        }

        for stream in listener.incoming() { //Wait for players
            match stream {
                Ok(stream) => {
                    self.accept_client_stream(stream);
                    count += 1;
                },
                Err(_) => {
                    self.log_sender.send("Unknown client detected.".to_string());
                }
            }

            if count >= self.player_limit {
                if !cfg!(test) {
                    self.log_sender.send("Player limit reached. The game will start soon!".to_string());
                }
                return;
            }
        }
    }
    pub fn accept_client_stream(&self, mut stream: net::TcpStream) {
        let json = "{".to_string() + &format!(r#""counter":{}"#,(self.comn_prov.lock().unwrap().clients_count())) + "}|";
        // tell a client id to the client.

        match stream.write(&json.into_bytes()) {
            Ok(_) => {
                if !cfg!(test) {
                    self.log_sender.send(format!("player joined! player details : {:?}", stream));
                }
                self.join_client_stream(stream);
            }
            Err(_) => {
                self.log_sender.send("error occured. this stream will ignore".to_string());
            }
        }
    }
    pub fn join_client_stream(&self, stream: net::TcpStream) {
        //self.clients.lock().unwrap().push(stream);
        self.comn_prov.lock().unwrap().push_client(stream, "0,0,0".to_string());

        self.player_info_initialize_in_map();
    }
    pub(super) fn player_info_initialize_in_map(&self) {
        self.game.lock().unwrap().map_proc.lock().unwrap().players.push(GameClient::default());
    }

    pub(super) fn wait_and_prepare_communication(&mut self) {
        // wait client effect
       
        for i in 0..self.player_limit {
            let msg = self.comn_prov.read_stream_buffer_at(i);
            if msg != "END_EFFECT\n" {
                panic!("Invalid message in wait client effect. data is {}", msg);
            }
        }
        if !cfg!(test) {
            self.log_sender.send("The game can start".to_string());
        }
    }
}

impl Drop for GameController {
    fn drop(&mut self) {
        println!("Game crushed");
    }
}


