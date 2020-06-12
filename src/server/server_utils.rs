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
    }
};

use super::{
    GameController,
    communication::{
        CommunicationProviderTrait,
        CommunicationProvider,
        BufStream,
        read_by_buffer,
    },
};

impl GameController {
    pub fn new(game: Game) -> GameController {
        GameController{
            player_limit: game. number_of_player,
            game: Arc::new(Mutex::new(game)),
            timer: LoopTimer::new(),
            comn_prov: Arc::new(Mutex::new(CommunicationProvider::new())),
        }
    }
    
    pub(super) fn wait_until_clients_connection(&mut self) {
        //let address = "2400:4051:99c2:58f0:11a4:53a7:248:a471:5522";
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
        let json = "{".to_string() + &format!(r#""counter":{}"#,(self.comn_prov.lock().unwrap().clients.len())) + "}|";
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
        let mut prov_ptr = self.comn_prov.lock().unwrap();
        prov_ptr.clients.push(Arc::new(Mutex::new(BufStream::new(&stream))));
        prov_ptr.network_buffer.push(Arc::new(Mutex::new("0, 0, 0".to_string())));

        self.player_info_initialize_in_map();
    }
    pub(super) fn player_info_initialize_in_map(&self) {
        self.game.lock().unwrap().map_proc.lock().unwrap().players.push(GameClient::default());
    }

    pub(super) fn wait_and_prepare_communication(&mut self) {
        // wait client effect
        
        let client_count = self.player_limit;
        let mut receivers = vec![];
        for i in 0..client_count {
            let stream_cloned = self.comn_prov.lock().unwrap().clients[i].clone();
            let (sender, receiver) = mpsc::channel();
            receivers.push(receiver);
            thread::spawn(move || {
                sender.send(read_by_buffer(stream_cloned)).unwrap();
            });
        }

        for r in receivers {
            match r.recv() {
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

        self.start_reading_coordinate();
    }

}

impl Drop for GameController {
    fn drop(&mut self) {
        println!("Game crushed");
    }
}


/*
#[allow(unused)]
pub fn print_typename<T>(_: T) {
    println!("type = {}", std::any::type_name::<T>());
}
*/


