
mod front;
mod game_parents;
pub mod server_interface;
mod server_utils;
pub mod communication;
mod data_factory;
mod worker;

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
    time::LoopTimer,
};


pub struct GameController {
    pub comn_prov: Arc<Mutex<CommunicationProvider>>,
    pub player_limit: usize,
    pub game: Arc<Mutex<Game>>,
    pub timer: LoopTimer, //This is doing tasks per the time
}




