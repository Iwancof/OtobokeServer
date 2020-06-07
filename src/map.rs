#![allow(unused)]

use std::ops::Sub;
use std::fs;
use std::sync::{Arc,Mutex};
use std::fmt;

use super::{
    serde_derive,
    serde,
    serde_json,
};

pub static UNIT_SIZE: f32 = 1.05;

pub struct MapInfo {
    pub width : usize,
    pub height :usize,
    pub field : Vec<Vec<i32>>,
}

pub struct MapProcAsGame {
    pub map: MapInfo,
    pub players: Vec<GameClient>, 
    pub pacman: QuanCoord,
    pub pm_target: usize,
    pub pm_inferpoints: Vec<QuanCoord>,
    pub pm_state: PMState,
    pub pm_prev_place: QuanCoord,
    pub paced_collection: Arc<Mutex<Vec<QuanCoord>>>,
}

#[derive(Copy, Clone)]
pub struct GameClient {
    pub coord: QuanCoord,
    //pub id: i32,
}

pub enum PMState {
    Normal,
    Powered,
}


#[derive(Copy, Clone)]
pub struct QuanCoord {
    pub x: i32,
    pub y: i32,
}

#[test]
fn serialize_test() {
    let x = RawCoord {
        x: 1.1,
        y: 2.2,
        z: 3.3,
    };
    let result = serde_json::to_string(&x).unwrap();
    println!("the result is {}", result);
    assert_eq!(1, 2);
}

#[derive(Copy, Clone, Serialize)]
pub struct RawCoord {
    // for use communicate with clients(Unity)
    pub x: f32,
    pub y: f32,
    pub z: f32, // clients(Unity) send z data.
}

impl GameClient {
    pub fn default() -> Self {
        Self {
            coord: QuanCoord::default(),
        }
    }
}

impl MapInfo {
    pub fn build_by_string(map_data: String) -> Self {
        let row : Vec<(usize,String)>
            = map_data.split('\n').map(|s| s.to_string()).enumerate().collect();
        
        let hei = row.len();
        let wid = row[0].1.len();

        println!("Map created. details, {},{} ",hei - 1,wid - 1);

        let mut map_tmp : Vec<Vec<i32>> = vec![vec![0;hei];wid];

        for (i,element) in row {
            //&element[0..(5)].to_string().chars().enumerate().for_each(|(j,nest)| map_tmp[j][i] = nest as i32 - 48);
            element.chars().enumerate().for_each(|(j,nest)| {
                if j < wid  { map_tmp[j][i] = nest as i32 - 48; } 
            });
        }

        Self {
            width: wid - 1,
            height: hei - 1, 
            field: map_tmp,
        }
    }
    pub fn build_by_filename(file_name: String) -> Self {
        let result = fs::read_to_string(&file_name);
        let map_data: String;
        match result {
            Ok(map_data) => {
                Self::build_by_string(map_data)
            },
            Err(_) => {
                panic!("Could not read file {}.", file_name)
            }
        }
    }
    pub fn show_map(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{}",self.field[x][y]);
            }
            println!("");
        }
    }
    pub fn acs_onvoid(&self, x: i32, y: i32) -> i32 {
        // if out of bounds, this will return 1(wall)
        if x < 0 || y < 0 || self.width as i32 <= x || self.height as i32 <= y {
            1
        } else {
            self.field[x as usize][y as usize]
        }
    }
    pub fn count_at(&self, x: i32, y: i32) -> i32 {
        let mut ret = 0;
        for dx in vec![-1, 1] {
            if self.acs_onvoid(x + dx, y) != 1 {
                // not wall
                ret += 1;
            }
        }
        for dy in vec![-1, 1] {
            if self.acs_onvoid(x, y + dy) != 1 {
                ret += 1;
            }
        }
        ret
    }
    pub fn get_inferpoints(&self) -> Vec<QuanCoord> {
        let mut ret = vec![];
        for x in 0..self.width as i32 {
            for y in 0..self.height as i32 {
                if self.acs_onvoid(x, y) != 1 && 3 <= self.count_at(x, y) {
                    ret.push(QuanCoord {
                        // (x, y) is "index" value. but we receive data which system's base is at bottom.
                        // so, we must convert system.
                        x: x,
                        y: self.height as i32 - y - 1,
                    });
                }
            }
        }
        ret
    }
    pub fn map_to_string(&self) -> String {
        let mut ret = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                ret += &self.field[x][y].to_string();
            }
            ret += ",";
        }
        ret
    }
}
impl MapProcAsGame {
    pub fn new(map: MapInfo, player_number: usize) -> Self {
        Self {
            pm_inferpoints: map.get_inferpoints(),
            map: map,
            players: vec![],
            //pacman: QuanCoord::default(),
            // dont erase.
            pacman: QuanCoord{ x: 0, y: 10 },
            pm_target: 0,
            pm_state: PMState::Normal,
            pm_prev_place: QuanCoord::default(),
            paced_collection: Arc::new(Mutex::new(vec![])),
        }
    }
}


impl RawCoord {
    fn quantize(&self) -> QuanCoord {
        QuanCoord {
            x: ((self.x - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
            y: ((self.y - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
        }
    }
    fn transform_json(&self) -> String {
        let mut ret = String::new();
        ret += "{";
        ret += &format!(
            r#""x":"{}","y":"{}","z":"{}""#, 
            self.x, self.y, self.z
        );
        ret += "}";
        ret
    }
}

impl QuanCoord {
    pub fn anti_quantize(&self) -> RawCoord {
        RawCoord {
            x: self.x as f32 * UNIT_SIZE,
            y: self.y as f32 * UNIT_SIZE,
            z: 0.,
        }
    }
    pub fn dist(a: QuanCoord, b: QuanCoord) -> f32 {
        (
            (a.x as f32 - b.x as f32).powf(2.) +
            (a.y as f32 - b.y as f32).powf(2.)
        ).sqrt()
    }
    pub fn default() -> Self {
        Self {
            x: 0,
            y: 0,
        }
    }
}

impl Sub for QuanCoord {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl fmt::Display for QuanCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl PartialEq for QuanCoord {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}



