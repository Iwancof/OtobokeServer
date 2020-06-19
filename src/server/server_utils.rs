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
    time::LoopTimer,
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
            timer: LoopTimer::new(),
            comn_prov: Arc::new(Mutex::new(CommunicationProvider::new())),
            conduc: WorkerConductor::new(snd),
        }
    }
    
    pub(super) fn wait_until_clients_connection(&mut self) {
        //let address = "2400:4051:99c2:58f0:11a4:53a7:248:a471:5522";
        //let address = "192.168.1.7:5522";
        let address = "192.168.1.7:5522";
        let listener = net::TcpListener::bind(address).unwrap(); //Create Listener
        let mut count = 0;

        println!("Server started listening at {}", address);

        for stream in listener.incoming() { //Wait for players
            match stream {
                Ok(stream) => {
                    self.accept_client_stream(stream);
                    count += 1;
                },
                Err(_) => {
                    println!("Unknown client detected.");
                }
            }

            if count >= self.player_limit {
                println!("Player limit reached. The game will start soon!");
                return;
            }
        }
    }
    pub fn accept_client_stream(&self, mut stream: net::TcpStream) {
        let json = "{".to_string() + &format!(r#""counter":{}"#,(self.comn_prov.lock().unwrap().clients_count())) + "}|";
        // tell a client id to the client.

        match stream.write(&json.into_bytes()) {
            Ok(_) => {
                println!("player joined! player details : {:?}", stream);
                self.join_client_stream(stream);
            }
            Err(_) => {
                println!("error occured. this stream will ignore");
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
    }

}

impl Drop for GameController {
    fn drop(&mut self) {
        println!("Game crushed");
    }
}


