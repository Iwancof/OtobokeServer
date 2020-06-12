#![allow(unused)]
#![allow(dead_code)]

mod server;
mod game;
mod network;
mod time;
mod algorithm;
mod map;
mod json;

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
};

fn main() { //For one game
    println!("\x1b[2J");
    println!("\x1b[0;0H");

    let args: Vec<String> = env::args().collect();
    let mut map_path = args[0].trim_matches(|c| c != '\\').to_string();
    map_path += "..\\..\\maps\\default_map";

    println!("Use default map. {}",map_path);

    let player_number = 1;

    let mut map = map::MapInfo::build_by_filename(map_path);
    // pure map info

    let mut game = game::Game::new(map, player_number);
    // make instance with map and number.

    let mut g = server::GameController::new(game);
    // make server call GameController
    
    g.show_game_details();
    println!("Game initialized");

    g.wait_for_players();
    // wait players
    
    g.game_initialize();

    g.distribute_map();
    // distribute map to clients.
    
    g.wait_and_prepare_communication();
    // wait effect end and start coordinates.

    g.start_game();
    // start!
    

    println!("Game end");
}

fn print_on(msg: String, wos: usize, hos: usize) {
    print!("\x1b[{};{}H{}", hos, wos, msg);
}


