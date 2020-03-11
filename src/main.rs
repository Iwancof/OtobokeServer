mod server;
mod game;
use std::thread;
use std::time::Duration;

fn main() { //For one game
    println!("\x1b[2J");
    println!("\x1b[0;0H");

    let map = game::Map::create_by_filename("../maps/default_map".to_string());
    
    //map.show_map();
    //println!("{}",game_instance.map_to_String());

    open_server(map);
}

fn open_server(map : game::Map) {
    let mut g = server::GameController::new(map);
    println!("Game initialized");

    g.wait_for_players();
    //g.initialize_players(); //distribute map and etc...
    thread::sleep(Duration::from_millis(3000));

    g.start_game();
}

