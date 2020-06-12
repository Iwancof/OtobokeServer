

use super::{
    GameController,
    communication::{
        CommunicationProviderTrait,
    },
};


impl GameController {
    pub fn server_flow_tmp(&mut self) {
        // TODO: remove this.
    
        self.show_game_details();
        println!("Game initialized");

        self.wait_until_clients_connection();
        // wait players
        
        self.game_initialize();

        self.distribute_map();
        // distribute map to clients.
        
        self.wait_and_prepare_communication();
        // wait effect end and start coordinates.

        self.start_game();
        // start!
    }

    pub fn show_game_details(&self) {
        println!("Player limit : {}",self.player_limit);
    }

    pub fn start_game(&mut self) {
        println!("GameReady!!");
        self.announce_wrap("GameReady|".to_string());

        let cloned_network_buffer = self.comn_prov.lock().unwrap().network_buffer.clone();
        let cloned_game = self.game.clone();
        let cloned_prov = self.comn_prov.clone();
        let client_count = self.player_limit;

        self.timer.subscribe(Box::new(move || {
            for i in 0..client_count {
                let player_lastest_message = cloned_network_buffer[i].lock().unwrap().clone();
                // get lastest player info
                //println!("\x1b[10;0Hclient[{}] at {}",i,&player_lastest_message);

                let parse_result = Self::parse_client_info(player_lastest_message);
                if parse_result.len() == 3 {
                    cloned_game.lock().unwrap().update_coordinate(i, parse_result);
                }
            };
            let mut received_data = cloned_game.lock().unwrap().coordinate_to_json();
            cloned_prov.send(received_data);
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
