#![allow(unused)]

use std::fs;
use std::fmt;
use std::sync::{Arc,Mutex};
use std::time::{Duration,SystemTime,Instant};
use std::thread;

use std::sync::mpsc::{
    SyncSender,
};
use super::{
    map::{
        MapProcAsGame,
        MapInfo,
        UNIT_SIZE,
        coord::{
            QuanCoord,
            RawCoord,
        },
    },
    json::{
        json_build,
        json_build_vec,
    },
    time::{
        WorkerConductor,
    },
};


pub struct Game {
    pub map_proc: Arc<Mutex<MapProcAsGame>>,
    pub pm_score: i32,
    pub number_of_player: usize,
    pub game_id: i32,
    pub snd: SyncSender<String>,
}

impl Game {
    pub fn new(map: MapInfo, num_of_player: usize, snd: SyncSender<String>) -> Self {
        Self {
            map_proc: Arc::new(Mutex::new(MapProcAsGame::new(map, snd.clone()))),
            pm_score: 0,
            number_of_player: num_of_player,
            game_id: -1,
            snd: snd.clone(),
        }
    }
    pub fn move_pacman_wrap(&mut self) {
        self.map_proc.lock().unwrap().move_pacman();
    }
    pub fn coordinate_to_json_pacman(&self) -> String {
        let ret = json_build("PACMAN", "Pacman", &self.map_proc.lock().unwrap().pacman.anti_quantize()) + "|";
        //println!("result of 'coordinate_to_json_pacman' is {}", ret);
        ret
    }
    pub fn update_coordinate(&mut self, i: usize, v: Vec<f32>) {
        // the player index, the player coordinate vector(raw)
        //println!("Update coordinate"); 

        let coord_ptr = &mut self.map_proc.lock().unwrap().players[i];
        coord_ptr.raw_coord.x = v[0];
        coord_ptr.raw_coord.y = v[1];
        coord_ptr.raw_coord.z = v[2];
        coord_ptr.coord = coord_ptr.raw_coord.quantize();
    }
    pub fn coordinate_to_json(&self) -> String {
        let ret = json_build_vec("PLAYER", "Coordinate", 
                       &self.map_proc.lock().unwrap().players.
                       iter().map(|x| x.raw_coord).collect()) + "|";
        //println!("result of 'coordinate_to_json' is {}", ret);
        ret
    }
}

pub fn paced_vec_to_string(v : Vec<QuanCoord>) -> String {
    if v.len() == 0 { return "".to_string(); }
    let mut ret = r#"PACCOL;{"Coordinate":["#.to_string();
    for i in 0..v.len() - 1 {
        ret += r#"{"x":""#;
        ret += &format!("{}", v[i].x);
        ret += r#"","y":""#;
        ret += &format!("{}", v[i].y);
        ret += r#"","z":""#;
        ret += &format!("{}", 0);
        ret += r#""},"#;
    }
    ret += r#"{"x":""#;
    ret += &format!("{}", v[v.len() - 1].x);
    ret += r#"","y":""#;
    ret += &format!("{}", v[v.len() - 1].y);
    ret += r#"","z":""#;
    ret += &format!("{}", 0);
    ret += r#""}]}|"#;

    ret
}


