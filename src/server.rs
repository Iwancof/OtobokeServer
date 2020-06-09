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

use super::{
    game::Game,
    network::{
        CommunicationProvider,
        start_reading_coordinate,
        read_by_buffer,
        parse_client_info,
    },
    time::LoopTimer,
    game,
    map::{
        self,
        GameClient,
    }
};

pub struct GameController {
    pub comn_prov: Arc<Mutex<CommunicationProvider>>,
    pub player_limit: usize,
    pub game: Arc<Mutex<Game>>,
    pub timer: LoopTimer, //This is doing tasks per the time
}

impl GameController {
    pub fn new(game: Game) -> GameController {
        GameController{
            player_limit: game. number_of_player,
            game: Arc::new(Mutex::new(game)),
            timer: LoopTimer::new(),
            comn_prov: Arc::new(Mutex::new(CommunicationProvider::new())),
        }
    }
    
    pub fn show_game_details(&self) {
        println!("Player limit : {}",self.player_limit);
    }

    pub fn wait_for_players(&mut self) {
        let address = "192.168.2.129:5522";
        let listener = net::TcpListener::bind(address).unwrap(); //Create Listener
        let mut count = 0;

        println!("Server started listening at {}", address);

        for stream in listener.incoming() { //Wait for players
            match stream {
                Ok(stream) => { self.player_join_initialize(stream);count += 1; },
                Err(_) => { println!("Unknown client detected.") }
            }

            if count >= self.player_limit {
                println!("Player limit reached. The game will start soon!");
                return;
            }
        }
    }
    pub fn register_player(&self) {
        self.game.lock().unwrap().map_proc.lock().unwrap().players.push(GameClient::default());
    }

    pub fn distribute_map(&mut self) {
        //let map_data = self.map.lock().unwrap().map_to_string();
        let map_data = self.game.lock().unwrap().map_proc.lock().unwrap().map.map_to_string() + "|";
        self.announce_wrap(map_data);
        self.announce_wrap(
            r#"{"value":""#.to_string() + &(self.player_limit.to_string()) + r#""}|"#);
        println!("map disributed");
    }
   
    pub fn start_game(&mut self) {
        self.announce_wrap("StartGame|".to_string());

        let clone_clients_for_announce_pac_coordinate = 
            self.comn_prov.lock().unwrap().clients.clone();
        let clone_clients_for_announce_bait_info = 
            self.comn_prov.lock().unwrap().clients.clone();
        let clone_game_for_announce_pac_coordinate = self.game.clone();
        let clone_game_for_announce_bait_info = self.game.clone();

        self.timer.subscribe(Box::new(move || { //Per 0.2 seconds, Program executes this closure.
            clone_game_for_announce_pac_coordinate.lock().unwrap().move_pacman_wrap();
            let msg = &clone_game_for_announce_pac_coordinate.lock().unwrap().coordinate_to_json_pacman().into_bytes();
                //Pacman coordinate convert to json here.
            for client_arc in &clone_clients_for_announce_pac_coordinate {
                match client_arc.lock() {
                    Ok(mut client) => {
                        client.write(msg).expect("Could now send pac coordinate"); //And send to clients. (Can't use announce methot)
                    
                    },
                    Err(_) => {
                        println!("Could not send pacman coordiante for client");
                    }
                }
            }
        }),
        200 //Time span
        );
        self.timer.subscribe(Box::new(move || {
            let msg = game::paced_vec_to_string(clone_game_for_announce_bait_info.lock().unwrap().get_paced_coordinates_as_raw()).into_bytes();
            
            for client_arc in &clone_clients_for_announce_bait_info {
                match client_arc.lock() {
                    Ok(mut client) => {
                        client.write(&msg).expect("Could not send bait info"); //And send to clients. (Can't use announce methot)
                    },
                    Err(_) => {
                        println!("Could not send pacman coordiante for client");
                    }
                }
            }
            clone_game_for_announce_bait_info.lock().unwrap().clear_paced_collection();
        }),200);

        self.distribute_map();

        let buffer_streams: Vec<Arc<Mutex<BufStream>>> // USAGE: access by index and mutex proc
            = self.comn_prov.lock().unwrap().clients.iter().map(|c| // each clients
                match c.lock() {
                    Ok(client) => {
                        Arc::new(
                            Mutex::new(
                                BufStream::new(&client) // make stream from clients
                                )
                            )
                    },
                    Err(_) => {
                        panic!("Error occured in initialize network");
                    }
                }
                ).collect();
       
        
        let client_count = self.player_limit;

        // wait client effect
        let mut receivers = vec![];
        for i in 0..client_count {
            let cloned : Arc<Mutex<BufStream>> = buffer_streams[i].clone();
            let (sender, receiver) = mpsc::channel();
            receivers.push(receiver);
            thread::spawn(move || {
                sender.send(read_by_buffer(cloned)).unwrap();
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

        println!("GameReady!!");
        self.announce_wrap("GameReady|".to_string());

        start_reading_coordinate(
            &buffer_streams, 
            &mut self.comn_prov.lock().unwrap().network_buffer.clone()
        );

        let cloned_network_buffer = self.comn_prov.lock().unwrap().network_buffer.clone();
        let cloned_game = self.game.clone();
        let cloned_prov = self.comn_prov.clone();

        self.timer.subscribe(Box::new(move || {
            for i in 0..client_count {
                let player_lastest_message = cloned_network_buffer[i].lock().unwrap().clone();
                // get lastest player info
                //println!("\x1b[10;0Hclient[{}] at {}",i,&player_lastest_message);

                let parse_result = parse_client_info(player_lastest_message);
                if parse_result.len() == 3 {
                    cloned_game.lock().unwrap().update_coordinate(i, parse_result);
                }
            };
            let received_data = cloned_game.lock().unwrap().coordinate_to_json();
            cloned_prov.lock().unwrap().announce_message(received_data);
        }), 50);

        self.timer.start(); //Start doing tasks
        
        loop { //main game loop 
            /*
            let mut server_stdin = String::new();
            io::stdin().read_line(&mut server_stdin);

            if server_stdin == "stop" {
                exit(0);
            }
            */
        }
    }
}

impl Drop for GameController {
    fn drop(&mut self) {
        println!("Game crushed");
    }
}

pub struct BufStream {
    pub rd: BufReader<TcpStream>,
    pub wr: BufWriter<TcpStream>,
}

impl BufStream {
    pub fn new(stream: &TcpStream) -> BufStream {
        BufStream{
            rd: BufReader::new(
                stream.try_clone().unwrap()),
            wr: BufWriter::new(
                stream.try_clone().unwrap())}
    }
}

#[allow(unused)]
pub fn print_typename<T>(_: T) {
    println!("type = {}", std::any::type_name::<T>());
}

