#![allow(unused)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod server;
mod game;
mod time;
mod map;
mod json;
mod system_observer;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::env;
use map::MapInfo;
use std::sync::{
    Arc,
    Mutex,
    mpsc::{
        Sender,
        Receiver,
        sync_channel,
    }
};
use server::{
    communication::{
        CommunicationProviderTrait,
        CommunicationProvider,
    },
};

fn main() {
    println!("OTOBOKE SERVER STARTING NOW!!");

    let (snd, rcv) = sync_channel(16);

    let args: Vec<String> = env::args().collect();
    let mut map_path = args[0].trim_matches(|c| c != '\\').to_string();
    map_path += "..\\..\\maps\\default_map";

    println!("Use default map. {}",map_path);

    let player_number = 1;

    let mut map = map::MapInfo::build_by_filename(map_path);
    // pure map info

    let mut game = game::Game::new(map, player_number);
    // make instance with map and number.

    let mut g = server::GameController::new(game, snd);
    // make server call GameController

    g.start_server();

    println!("Game end");
}

fn print_on(msg: String, wos: usize, hos: usize) {
    print!("\x1b[{};{}H{}", hos, wos, msg);
}


