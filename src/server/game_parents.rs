
use std::{
    io::{
        Write,
    },
    time::{
        Duration,
    },
};

use super::{
    GameController,
};

use super::{
    worker::{
        WorkerResult,
    },
};

use super::communication::CommunicationProviderTrait;

impl GameController {
    pub(super) fn game_initialize(&mut self) {
        self.announce_wrap("StartGame|".to_string());
        self.set_communication_provider_to_map_proc();

        let clone_clients_for_announce_pac_coordinate = self.comn_prov.clone(); // Copy
        let clone_clients_for_announce_bait_info = self.comn_prov.clone();
        let clone_game_for_announce_pac_coordinate = self.game.clone();
        let clone_game_for_announce_bait_info = self.game.clone();

        /*
        self.timer.subscribe(Box::new(move || { //Per 0.2 seconds, Program executes this closure.
            clone_game_for_announce_pac_coordinate.lock().unwrap().move_pacman_wrap();
            let msg = clone_game_for_announce_pac_coordinate.lock().unwrap().coordinate_to_json_pacman();
                //Pacman coordinate convert to json here.
            clone_clients_for_announce_pac_coordinate.lock().unwrap().send(msg);
        }),
        200 //Time span
        );
        */
        self.conduc.add_task(Box::new(move || {
            clone_game_for_announce_pac_coordinate.lock().unwrap().move_pacman_wrap();
            let msg = clone_game_for_announce_pac_coordinate.lock().unwrap().coordinate_to_json_pacman();
                //Pacman coordinate convert to json here.
            clone_clients_for_announce_pac_coordinate.lock().unwrap().send(msg);
            WorkerResult::Complete
        }), Duration::from_millis(200));

        //self.conduc.start();


    }
    pub(super) fn set_communication_provider_to_map_proc(&self) {
        self.game.lock().unwrap().map_proc.lock().unwrap().comn_prov = Some(self.comn_prov.clone());
    }

    pub(super) fn distribute_map(&mut self) {
        //let map_data = self.map.lock().unwrap().map_to_string();
        let map_data = self.game.lock().unwrap().map_proc.lock().unwrap().map.map_to_string() + "|";
        self.announce_wrap(map_data);
        self.announce_wrap(
            r#"{"value":""#.to_string() + &(self.player_limit.to_string()) + r#""}|"#);
        println!("map disributed");
    }

}
