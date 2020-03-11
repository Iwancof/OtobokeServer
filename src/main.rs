mod server;
mod game;
use std::thread;
use std::time::Duration;
use std::env;

fn main() { //For one game
    println!("\x1b[2J");
    println!("\x1b[0;0H");

    let args : Vec<String> = env::args().collect();
    let mut map_path = args[0].trim_matches(|c| c != '\\').to_string();
    map_path += "..\\..\\..\\maps\\default_map";

    println!("Use default map. {}",map_path);

    let map = game::Map::create_by_filename(map_path);
    
    //map.show_map();
    //println!("{}",game_instance.map_to_String());

    open_server(map);
}

fn open_server(map : game::Map) {
    let mut g = server::GameController::new(map);
    println!("Game initialized");

    g.wait_for_players();
    //g.initialize_players(); //distribute map and etc...
    //thread::sleep(Duration::from_millis(3000));

    g.start_game();
}

