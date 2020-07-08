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
    pub fn start_server(&mut self) {
        self.show_game_details();
        self.log_sender.send("Game initialized".to_string());

        self.wait_until_clients_connection("2400:4051:99c2:58f0:11a4:53a7:248:a471:5522");
        // wait players

        self.game_initialize();

        self.distribute_map();
        // distribute map to clients.
        
        self.wait_and_prepare_communication();
        // wait effect end and start coordinates.

        self.lanch_game();
        // start!
    }
    pub fn server_flow_to_test(&mut self) {
        self.wait_until_clients_connection("127.0.0.1:5522");
        self.game_initialize();
        self.distribute_map();
        self.wait_and_prepare_communication();
        
        self.announce_wrap("THIS IS TEST MESSAGE|".to_string());
    }

    pub fn show_game_details(&self) {
        self.log_sender.send(format!("Player limit : {}",self.player_limit));
    }

    pub fn lanch_game(&mut self) {
        self.log_sender.send("GameReady!!".to_string());
        self.announce_wrap("GameReady|".to_string());
        self.start_reading_coordinate();

        let cloned_prov_network_buf = self.comn_prov.clone();
        let cloned_game = self.game.clone();
        let cloned_prov = self.comn_prov.clone();
        let client_count = self.player_limit;
        
        self.conduc.add_task(Box::new(move || {
            for i in 0..client_count {
                let player_lastest_message = cloned_prov_network_buf.get_buffer_at(i);
                // get lastest player info

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
    }
}
