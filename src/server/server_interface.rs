use std::{
    time::{
        Duration,
    },
};
use super::{
    worker::{
        WorkerResult,
    },
};

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
        self.start_reading_coordinate();

        let cloned_prov_network_buf = self.comn_prov.clone();
        let cloned_game = self.game.clone();
        let cloned_prov = self.comn_prov.clone();
        let client_count = self.player_limit;

        /*
        self.timer.subscribe(Box::new(move || {
            for i in 0..client_count {
                let player_lastest_message = cloned_prov_network_buf.get_buffer_at(i);
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
        */
        self.conduc.add_task(Box::new(move || {
            for i in 0..client_count {
                let player_lastest_message = cloned_prov_network_buf.get_buffer_at(i);
                // get lastest player info
                //println!("\x1b[10;0Hclient[{}] at {}",i,&player_lastest_message);

                let parse_result = Self::parse_client_info(player_lastest_message);
                if parse_result.len() == 3 {
                    cloned_game.lock().unwrap().update_coordinate(i, parse_result);
                }
            };
            let mut received_data = cloned_game.lock().unwrap().coordinate_to_json();
            cloned_prov.send(received_data);

            WorkerResult::Complete
        }), Duration::from_millis(50));

        self.conduc.start(); //Start doing tasks
        
        loop { //main game loop 
            if self.end_game {
                // or... drop this?

                break;
            }
        }
    }
    fn stop(&mut self) {
        self.end_game = true;
        self.conduc.stop();
    }

}
