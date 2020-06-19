
mod game_control;
pub mod api;
mod internal_procs;
pub mod communication;
pub mod worker;
mod test;

//       Clients
//          |
//     /--front------
//     |            |
// game_parents   server_interface<--project_main
//   |       |      |
// Games    server_utils---worker
//             |
//       communication
//             |
//        data_factory
 
use std::{
    sync::{
        Arc,
        Mutex,
    },
};

use communication::{
    CommunicationProviderTrait,
    CommunicationProvider,
};

use crate::{
    game::Game,
    time::{
        WorkerConductor,
    }
};


pub struct GameController {
    comn_prov: Arc<Mutex<CommunicationProvider>>,
    player_limit: usize,
    game: Arc<Mutex<Game>>,
    conduc: WorkerConductor,
    end_game: bool,
}




