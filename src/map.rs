#![allow(unused)]

use std::ops::Sub;
use std::fs;
use std::sync::{
    Arc, 
    Mutex, 
    atomic::AtomicBool,
    mpsc::{
        Sender,
    }
};
use std::fmt;
use std::collections::HashMap;

use super::{
    serde_derive,
    serde,
    serde_json,
    time,
};

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
    pub paced_collection: Arc<Mutex<Vec<QuanCoord>>>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GameClient {
    pub coord: QuanCoord,
    pub raw_coord: RawCoord,
}

#[derive(Clone)]
pub enum PMState {
    Normal,
    Powered(Sender<time::Message>), // this sender is to stop thread. 
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct QuanCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, Serialize, PartialEq)]
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
            raw_coord: RawCoord::default(),
        }
    }
}

impl MapInfo {
    pub fn build_by_string(map_data: String) -> Self {
        // first, create Vec<String> split by "\n"
        let sliced_map: Vec<String> = map_data.split('\n').map(|s| s.to_string()).collect();
        
        // second, create Vec<Vec<char>>
        let char_map: Vec<Vec<char>> = sliced_map.iter().map(|s| s.chars().collect()).collect();

        // and, each Vec<char>'s char convert to map_chip integer.
        let mirror_map_data: Vec<Vec<i32>> = char_map.iter().take_while(|v| v.len() != 0).
            map(|vector| vector.iter().
                map(|c| *c as i32 - '0' as i32).take_while(|v| Self::is_map_chip(*v)).
                collect()
            ).collect();

        let h = mirror_map_data.len();
        if h == 0 {
            panic!("map data is empty");
        }

        let w = mirror_map_data[0].len();
        // check each width is w.
        if mirror_map_data.iter().any(|v| v.len() != w) {
            panic!("map data is not valid");
        }
        
        // now, map_data's usage is "mirror_map_data[y][x]". and transform to "map_data[x][y]"
        let mut map_data = vec![vec![0; h]; w];
        for x in 0..w {
            for y in 0..h {
                map_data[x][y] = mirror_map_data[y][x];
            }
        }
    
        println!("Map created. details, {},{} ", w, h);

        let mut uniques = HashMap::new();
        for x in 0..w {
            for y in 0..h {
                for e in &UNIQUE_ELEMENTS {
                    if map_data[x][y] == *e {
                        uniques.insert(*e, QuanCoord{x: x as i32, y: (h - y - 1) as i32});
                    }
                }
            }
        }

        Self {
            width: w,
            height: h, 
            field: map_data,
            unique_points: uniques,
        }
    }

    fn is_map_chip(v: i32) -> bool {
        0 <= v && v <= 10
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
    pub fn get_can_move_on(&self, now_pos: QuanCoord) -> Vec<QuanCoord> {
        let mut ret = vec![];
        for diff in &[-1, 1] {
            let checking_point = now_pos.plus_element_x(*diff).torus_form(self);
            if self.access_by_coord_game_based_system(checking_point) != 1 {
                ret.push(checking_point);
            }
            let checking_point = now_pos.plus_element_y(*diff).torus_form(self);
            if self.access_by_coord_game_based_system(checking_point) != 1 {
                ret.push(checking_point);
            }
        }
        ret
    }
    pub fn access_by_coord_game_based_system_mutref(&mut self, mut coord: QuanCoord) -> &mut i32 {
        // coord is game based system. so we must convert to index system
        coord = coord.torus_form(self); // convert to torus form
        coord.y = self.height as i32 - coord.y - 1;  // convert to index based system
        &mut self.field[coord.x as usize][coord.y as usize]
    }
    pub fn access_by_coord_game_based_system(&self, mut coord: QuanCoord) -> i32 {
        coord = coord.torus_form(self);
        coord.y = self.height as i32 - coord.y - 1;
        self.field[coord.x as usize][coord.y as usize]
    }
    pub fn access_by_coord_index_based_converted_system_mutref(&mut self, x: i32, y: i32) -> &mut i32 {
        let x = (x + self.width as i32) % self.width as i32;
        let y = (y + self.height as i32) % self.height as i32;
        &mut self.field[x as usize][y as usize]
    }
    pub fn access_by_coord_index_based_converted_system(&self, x: i32, y: i32) -> i32 {
        let x = (x + self.width as i32) % self.width as i32;
        let y = (y + self.height as i32) % self.height as i32;
        self.field[x as usize][y as usize]
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
            pacman: QuanCoord{ x: 24, y: 16 },
            pm_target: 0,
            pm_state: Arc::new(Mutex::new(PMState::Normal)),
            pm_prev_place: QuanCoord{ x: 25, y: 16 },
            paced_collection: Arc::new(Mutex::new(vec![])),
        }
    }
}


impl RawCoord {
    pub fn quantize(&self) -> QuanCoord {
        QuanCoord {
            x: ((self.x - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
            y: ((self.y - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
        }
    }
    pub fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl QuanCoord {
    pub fn anti_quantize(&self) -> RawCoord {
        RawCoord {
            x: self.x as f32 * UNIT_SIZE,
            y: (self.y as f32 + 1.) * UNIT_SIZE,
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
    pub fn plus_element_x(&self, x: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }
    pub fn plus_element_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }
    pub fn torus_form(&self, map_ptr: &MapInfo) -> Self {
        Self {
            x: (self.x as i32 + map_ptr.width as i32) % map_ptr.width as i32,
            y: (self.y as i32 + map_ptr.height as i32) % map_ptr.height as i32,
        }
    }
    pub fn distance_to_element(&self, x: i32, y: i32) -> f64 {
        (((self.x - x) as f64).powf(2.) + ((self.y - y) as f64).powf(2.)).sqrt()
    }
    pub fn distance_to_coord(&self, other: Self) -> f64 {
        (((self.x - other.x) as f64).powf(2.) + ((self.y - other.y) as f64).powf(2.)).sqrt()
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

fn create_map_mock() -> MapInfo {
    let map_string = "01010\n00000\n31014\n10110\n01110".to_string();
    // 01010 <- (4, 4)
    // 00000
    // 31014
    // 10110
    // 01110
    // ^
    // (0, 0)
    MapInfo::build_by_string(map_string)
}

#[test]
fn map_mock_valid_test() {
    let m = create_map_mock();
    assert_eq!(m.width, 5);
    assert_eq!(m.height, 5);
}

#[test]
fn field_access_test() {
    let map = create_map_mock();
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 0, y: 0}), 0);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 1, y: 1}), 0);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 1, y: 4}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 3, y: 1}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 3, y: 1}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 5, y: 2}), 3);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 6, y: 2}), 1);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 7, y: 2}), 0);
    assert_eq!(map.access_by_coord_game_based_system(QuanCoord{x: 8, y: 2}), 1);
}

#[test]
fn unique_field_element_search_test() {
    let map = create_map_mock();
    assert_eq!(*map.unique_points.get(&3).unwrap(), QuanCoord{x: 0, y: 2});
    assert_eq!(*map.unique_points.get(&4).unwrap(), QuanCoord{x: 4, y: 2});
}

#[test]
fn map_file_import_test() {
    let map = MapInfo::build_by_filename("C:\\Users\\Rock0x3FA\\OtobokeServer\\maps\\default_map".to_string());
    println!("({}, {})", map.width, map.height);
    //assert_eq!(0, 1);
}

#[test]
fn search_movable_point_test() {
    let map = create_map_mock();
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 2, y: 2}), &vec![QuanCoord{x: 2, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 2, y: 3}), &vec![QuanCoord{x: 1, y: 3}, QuanCoord{x: 2, y: 2}, QuanCoord{x: 2, y: 4}, QuanCoord{x: 3, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 4, y: 2}), &vec![QuanCoord{x: 0, y: 2}, QuanCoord{x: 4, y: 1}, QuanCoord{x: 4, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 0, y: 3}), &vec![QuanCoord{x: 0, y: 4}, QuanCoord{x: 0, y: 2}, QuanCoord{x: 1, y: 3}, QuanCoord{x: 4, y: 3}]));
    assert!(vec_group_eq(&map.get_can_move_on(QuanCoord{x: 4, y: 4}), &vec![QuanCoord{x: 0, y: 4}, QuanCoord{x: 4, y: 0}, QuanCoord{x: 4, y: 3}]));
}
// 01010 <- (4, 4)
// 00000
// 31014
// 10110
// 01110
// ^
// (0, 0)


#[test]
fn vec_group_eq_test() {
    let a = vec![1, 2, 3, 4];
    let b = vec![4, 3, 2, 1];
    let c = vec![1, 2, 3];
    let d = vec![1, 2, 3, 5];

    assert_eq!(vec_group_eq(&a, &a), true);
    assert_eq!(vec_group_eq(&a, &b), true);
    assert_eq!(vec_group_eq(&a, &c), false);
    assert_eq!(vec_group_eq(&a, &d), false);
    assert_eq!(vec_group_eq(&b, &a), true);
    assert_eq!(vec_group_eq(&b, &a), true);
    assert_eq!(vec_group_eq(&b, &b), true);
    assert_eq!(vec_group_eq(&b, &c), false);
    assert_eq!(vec_group_eq(&b, &d), false);
}

fn vec_group_eq<T: PartialEq>(v: &Vec<T>, w: &Vec<T>) -> bool {
    if v.len() != w.len() {
        return false;
    }
    !v.iter().any(|e| !w.iter().any(|x| *x == *e))
}

fn create_map_proc_as_game_mock() -> MapProcAsGame {
    MapProcAsGame {
        map: create_map_mock(), 
        players: vec![],
        pacman: QuanCoord::default(),
        pm_target: 0,
        pm_inferpoints: create_map_mock().get_inferpoints(),
        pm_state: Arc::new(Mutex::new(PMState::Normal)),
        pm_prev_place: QuanCoord::default(),
        paced_collection: Arc::new(Mutex::new(vec![])),
    }
}
// 01010 <- (4, 4)
// 00000
// 31014
// 10110
// 01110
// ^
// (0, 0)

#[test]
fn routed_next_point_test() {
    let mut mock = create_map_proc_as_game_mock();
    mock.pacman = QuanCoord{x: 1, y: 3};
    mock.pm_prev_place = QuanCoord{x: 0, y: 3};
    assert_eq!(mock.routed_next_point(mock.map.get_can_move_on(mock.pacman)), QuanCoord{x: 2, y: 3});

    mock.pacman = QuanCoord{x: 4, y: 1};
    mock.pm_prev_place = QuanCoord{x: 4, y: 2};
    assert_eq!(mock.routed_next_point(mock.map.get_can_move_on(mock.pacman)), QuanCoord{x: 4, y: 0});
}

#[test]
fn move_to_non_teleport_point_test() {
    let mut mock = create_map_proc_as_game_mock();
    mock.pacman = QuanCoord{x: 2, y: 2};
    mock.move_to(QuanCoord{x: 2, y: 3}).unwrap();
    assert_eq!(mock.pacman, QuanCoord{x: 2, y: 3});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 2, y: 2});

    mock.pacman = QuanCoord{x: 2, y: 2};
    mock.move_to(QuanCoord{x: 2, y: 3}).unwrap();
    assert_eq!(mock.pacman, QuanCoord{x: 2, y: 3});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 2, y: 2});

    mock.pacman = QuanCoord{x: 1, y: 1};
    mock.pm_prev_place = QuanCoord{x: -1, y: -1};
    let result = mock.move_to(QuanCoord{x: 1, y: 2}); // (1, 2) is wall. so expected return is Err( (1, 2)).
    assert_eq!(result, Err(QuanCoord{x: 1, y: 2}));
    assert_eq!(mock.pacman, QuanCoord{x: 1, y: 1}); 
    assert_eq!(mock.pm_prev_place, QuanCoord{x: -1, y: -1}); // and. not change.
}

#[test]
fn move_to_teleport_point_test() {
    let mut mock = create_map_proc_as_game_mock();
    mock.pacman = QuanCoord{x: 0, y: 1};
    mock.move_to(QuanCoord{x: 0, y: 2}).unwrap(); // go to '3' teleport point
    assert_eq!(mock.pacman, QuanCoord{x: 4, y: 2});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 0, y: 2});

    mock.pacman = QuanCoord{x: 4, y: 1};
    mock.move_to(QuanCoord{x: 4, y: 2}).unwrap(); // go to '3' teleport point
    assert_eq!(mock.pacman, QuanCoord{x: 0, y: 2});
    assert_eq!(mock.pm_prev_place, QuanCoord{x: 4, y: 2});
}

// 01010 <- (4, 4)
// 00000
// 31014
// 10110
// 01110
// ^
// (0, 0)


/*
    pub map: MapInfo,
    pub players: Vec<GameClient>, 
    pub pacman: QuanCoord,
    pub pm_target: usize,
    pub pm_inferpoints: Vec<QuanCoord>,
    pub pm_state: PMState,
    pub pm_prev_place: QuanCoord,
    pub paced_collection: Arc<Mutex<Vec<QuanCoord>>>,
*/
