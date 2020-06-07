#![allow(unused)]
#![allow(dead_code)]

mod server;
mod game;
mod network;
mod time;
mod algorithm;
mod map;

use std::env;
use map::MapInfo;

fn main() { //For one game
    println!("\x1b[2J");
    println!("\x1b[0;0H");

    let args: Vec<String> = env::args().collect();
    let mut map_path = args[0].trim_matches(|c| c != '\\').to_string();
    map_path += "..\\..\\maps\\default_map";

    println!("Use default map. {}",map_path);

    let l = 1; // players
    let mut map = map::MapInfo::build_by_filename(map_path);
    let mut game = game::Game::new(map, l);

    open_server(game);
}

fn open_server(game: game::Game) {
    let mut g = server::GameController::new(game);
    g.show_game_details();
    println!("Game initialized");

    g.wait_for_players();
    //g.initialize_players(); //distribute map and etc...
    //thread::sleep(Duration::from_millis(3000));

    g.start_game();
}

fn print_on(msg: String, wos: usize, hos: usize) {
    print!("\x1b[{};{}H{}", hos, wos, msg);
}

