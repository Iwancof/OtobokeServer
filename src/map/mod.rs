
use super::{
    time,
};

use std::{
    sync::{
        mpsc::{
            Sender,
        },
        Arc,
        Mutex,

    },
    collections::{
        HashMap,
    },
};

pub mod map;
pub mod coord;

use coord::{
    QuanCoord,
    RawCoord,
};

use crate::network::CommunicationProvider;

pub const UNIT_SIZE: f32 = 1.05;
pub const UNIQUE_ELEMENTS: [i32; 2] = [3, 4];
pub const PACMAN_POWERED_TIME: f64 = 8.0; // 8 sec

pub struct MapInfo {
    pub width: usize,
    pub height: usize,
    pub field: Vec<Vec<i32>>,
    pub unique_points: HashMap<i32, QuanCoord>,
}

pub struct MapProcAsGame {
    pub map: MapInfo,
    pub players: Vec<GameClient>, 
    pub pacman: QuanCoord,
    pub pm_target: usize,
    pub pm_inferpoints: Vec<QuanCoord>,
    pub pm_state: Arc<Mutex<PMState>>,
    pub pm_prev_place: QuanCoord,
    pub comn_prov: Option<Arc<Mutex<CommunicationProvider>>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct GameClient {
    pub coord: QuanCoord,
    pub raw_coord: RawCoord,
}

#[derive(Clone)]
pub enum PMState {
    Normal,
    Powered(Sender<time::Message>), // this sender is to stop thread. 
}

impl ToString for PMState {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        match *self {
            Self::Normal => "Normal".to_string(),
            Self::Powered(_) => "Powered".to_string(),
        }
    }
}









