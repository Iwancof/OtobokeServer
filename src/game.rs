#![allow(unused)]

use std::fs;
use std::fmt;
use std::sync::{Arc,Mutex};
use std::time::{Duration,SystemTime,Instant};
use std::thread;

use super::{
    map::{
        MapProcAsGame,
        MapInfo,
        QuanCoord,
        RawCoord,
        UNIT_SIZE,
    },
    json::{
        json_build,
        json_build_vec,
    }
};


pub struct Game {
    pub map_proc: Arc<Mutex<MapProcAsGame>>,
    pub pm_score: i32,
    pub number_of_player: usize,
    pub game_id: i32,
}

impl Game {
    pub fn new(map: MapInfo, num_of_player: usize) -> Self {
        Self {
            map_proc: Arc::new(Mutex::new(MapProcAsGame::new(map, num_of_player))),
            pm_score: 0,
            number_of_player: num_of_player,
            game_id: -1,
        }
    }
    pub fn move_pacman(&mut self) {
        self.map_proc.lock().unwrap().pacman.x += 1;
    }
    pub fn coordinate_to_json_pacman(&self) -> String {
        let ret = json_build("PACMAN", "Pacman", &self.map_proc.lock().unwrap().pacman.anti_quantize()) + "|";
        //println!("result of 'coordinate_to_json_pacman' is {}", ret);
        ret
    }
    pub fn update_coordinate(&mut self, i: usize, v: Vec<f32>) {
        // the player index, the player coordinate vector(raw)
        //println!("Update coordinate"); 

        let client_coord_ptr = &mut self.map_proc.lock().unwrap().players[i].coord;
        client_coord_ptr.x = ((v[0] - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32;
        client_coord_ptr.y = ((v[1] - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32;
    }
    pub fn coordinate_to_json(&self) -> String {
        let ret = json_build_vec("PLAYER", "Coordinate", 
                       &self.map_proc.lock().unwrap().players.
                       iter().map(|x| x.coord.anti_quantize()).collect()) + "|";
        //println!("result of 'coordinate_to_json' is {}", ret);
        ret
    }
    pub fn get_paced_coordinates_as_raw(&self) -> Vec<RawCoord> {
        //println!("Get paced coordinates");
        self.map_proc.lock().unwrap().
            paced_collection.lock().unwrap().
                iter().map(|x| x.anti_quantize()).collect()
    }
    pub fn clear_paced_collection(&mut self) {
        *self.map_proc.lock().unwrap().paced_collection.lock().unwrap() = vec![];
    }

}

pub fn paced_vec_to_string(v : Vec<RawCoord>) -> String {
    if v.len() == 0 { return "".to_string(); }
    let mut ret = r#"PACCOL;{"Coordinate":["#.to_string();
    for i in 0..v.len() - 1 {
        ret += r#"{"x":""#;
        ret += &format!("{}",v[i].x);
        ret += r#"","y":""#;
        ret += &format!("{}",v[i].y);
        ret += r#"","z":""#;
        ret += &format!("{}",v[i].z);
        ret += r#""},"#;
    }
    ret += r#"{"x":""#;
    ret += &format!("{}",v[v.len() - 1].x);
    ret += r#"","y":""#;
    ret += &format!("{}",v[v.len() - 1].y);
    ret += r#"","z":""#;
    ret += &format!("{}",v[v.len() - 1].z);
    ret += r#""}]}|"#;

    ret
}


