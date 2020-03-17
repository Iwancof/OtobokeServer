#![allow(unused)]

use std::fs;
use std::fmt;
use super::algorithm::{Set};

const SIZE : f32 = 1.05;
const TARGETS : [Set;5] = [
    Set{x:0,y:1},
    Set{x:1,y:0},
    Set{x:-1,y:0},
    Set{x:0,y:-1},
    Set{x:0,y:0},
];


pub struct Map {
    pub width : usize,
    pub height :usize,
    pub field : Vec<Vec<i32>>,
    pub players : Vec<Player>,
    pub players_on_map : Vec<PlayerOnMap>,
    pub pacman : PlayerOnMap,
    pub count : usize,
    pub target_index : usize,
    pub infer_points : Vec<PlayerOnMap>,
    pub prev_place : PlayerOnMap,
    pub invers : bool,
    pub pacman_score : i32,
}

impl Map {
    pub fn new(w : usize ,h : usize) -> Map {
        Map{
            width:w,
            height:h,
            field:vec![vec![0;h];w],
            players:vec![],
            players_on_map:vec![],
            pacman:PlayerOnMap::new(0,0,0),
            count:0,
            target_index:4,
            infer_points : vec![],
            prev_place : PlayerOnMap::new(0,0,0),
            invers : false,
            pacman_score : 0,
        }
    }

    pub fn create_by_map_string(map_data : String) -> Map {
        let row : Vec<(usize,String)> = map_data.split('\n').map(|s| s.to_string()).enumerate().collect();
        //dont handle all elements length is not equal. mendo-kusai

        let hei = row.len();
        let wid = row[0].1.len();

        println!("{},{}",hei - 1,wid - 1);

        let mut map_tmp : Vec<Vec<i32>> = vec![vec![0;hei];wid];
        for (i,element) in row {
            //&element[0..(5)].to_string().chars().enumerate().for_each(|(j,nest)| map_tmp[j][i] = nest as i32 - 48);
            element.chars().enumerate().for_each(|(j,nest)| {
                if j < wid  { map_tmp[j][i] = nest as i32 - 48; } 
            });
        }

        Map{
            width:wid - 1,
            height:hei - 1,
            field:map_tmp,
            players:vec![],
            players_on_map:vec![],
            pacman:PlayerOnMap::new(9,13,0),
            count:0,
            target_index:4,
            infer_points : vec![],
            prev_place : PlayerOnMap::new(9,14,0),
            invers : false,
            pacman_score : 0,
        }
    }
   
    pub fn create_by_filename(file_name : String) -> Map {
        let result = fs::read_to_string(&file_name);
        let map_data : String;
        match result {
            Ok(string) =>  {
                map_data = string;
            }
            Err(_) => {
                panic!("[Error]File {} is not exist.",&file_name);
            }
        }
        Map::create_by_map_string(map_data)
    }

    pub fn initialize(&mut self) {
        let tmp = self.find_infer_point();
        self.infer_points = self.convert_system(tmp);
    }

    pub fn show_map(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{}",self.field[x][y]);
            }
            println!("");
        }
    }

    pub fn map_to_string(&self) -> String { //String is Json
        let mut ret = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                ret += &self.field[x][y].to_string();
            }
            ret += ",";
        }
        ret + "|"
    }
    pub fn coordinate_to_json(&self) -> String {
        let mut ret = String::new();
        ret += r#"PLAYER;{"Coordinate":["#;
        for i in 0..self.players.len() - 1 {
            ret += &(self.players[i].coordinate_to_json() + ",");
        }
        ret += &(self.players[self.players.len() - 1].coordinate_to_json());
        ret += r#"]}|"#;
        ret
        /*
        ret += r#"],"Pacman":"#;
        ret += &self.pacman.coordinate_to_json();
        ret += "}|";
        ret
        */
    }
    pub fn coordinate_to_json_pacman(&self) -> String {
        let mut ret = String::new();
        ret += r#"PACMAN;{"Pacman":"#;
        ret += &(self.pacman.coordinate_to_json());
        ret += r#"}|"#;
        ret
              
    }

    pub fn print_onmap_coordinate(&self) {
        for e in &self.players {
            println!("{}",e.on_map_coordinate());
        }
    }
    pub fn print_map(&self,wos : usize,hos : usize) {
        let p : Vec<PlayerOnMap> = self.players.iter().map(|x| x.on_map_coordinate()).collect();
        for y in 0..self.height {
            for x in 0..self.width {
                for p in &p {
                    if x == p.x as usize && y == p.y as usize {
                        super::print_on("#".to_string(),x,self.height - y);
                    } else {
                        super::print_on(format!("{}",self.field[x][y]),x ,y);
                    }
                }
            }
        }
    }
    pub fn set_priority(&self) -> Vec<Set> {
        let mut ret = vec![];
        let target = &self.players_on_map[self.target_index];
        if self.pacman.x <= target.x {ret.push(Set::new(1,0));}
        else {ret.push(Set::new(-1,0));}
        if self.pacman.y <= target.y {ret.push(Set::new(0,1));}
        else {ret.push(Set::new(0,-1));}
        if target.x < self.pacman.x {ret.push(Set::new(1,0));}
        else {ret.push(Set::new(-1,0));}
        if target.y < self.pacman.y {ret.push(Set::new(0,1));}
        else {ret.push(Set::new(0,-1));}
        ret
    }
    
    pub fn infer_next_point(&self) -> PlayerOnMap {
        let can_moves = self.search_can_move_point();
        let dir_tmp = self.set_priority();

        //Target direction
        match can_moves.iter().map(|x| 
                      (x.clone(),match dir_tmp.iter().position(|y| (x.clone() - self.pacman.clone()).to_set() == *y) {
                                    Some(i) => i,
                                    None => 10,
                                }
                      )
            ).min_by_key(|x| x.1) {
                Some(i) => i.0,
                None => PlayerOnMap::new2(0,0),
            }
    }
            
    pub fn search_can_move_point(&self) -> Vec<PlayerOnMap> {
        let mut can_moves : Vec<PlayerOnMap> = vec![];
        for dx in vec![-1,1] {
            let (x,y) : (i32,i32) = (
                (self.pacman.x as i32 + dx) % self.width as i32,
                self.pacman.y as i32
                );
            if self.can_move(x,y) {
                can_moves.push(PlayerOnMap::new2(x,y));
            }
        }
        for dy in vec![-1,1] {
            let (x,y) : (i32,i32) = (
                self.pacman.x as i32 ,
                (self.pacman.y as i32 + dy) % self.height as i32
                );
            if self.can_move(x,y) {
                can_moves.push(PlayerOnMap::new2(x,y));
            }
        }
        can_moves
    }

    pub fn routed_next_point(&self) -> PlayerOnMap {
        let (mut new_x,mut new_y) = (0,0);
        let mut flag = false;
        for dx in vec![-1,1] {
            let (x,y) : (i32,i32) = (
                (self.pacman.x as i32 + dx) % self.width as i32,
                self.pacman.y as i32
                );
            if self.can_move(x,y) {
                if flag { panic!(r#""routed_next_point" must not be called in non-infer point"#); }
                new_x = x;
                new_y = y;
                flag = true;
            }
        }
        for dy in vec![-1,1] {
            let (x,y) : (i32,i32) = (
                self.pacman.x as i32,
                (self.pacman.y as i32 + dy) % self.height as i32
                );
            if self.can_move(x,y) {
                if flag { panic!(r#""routed_next_point" must not be called in non-infer point"#); }
                new_x = x;
                new_y = y;
                flag = true;
            }
        }
        if !flag { panic!(r#""routed_next_point" must not be called in end point"#); }
        PlayerOnMap::new2(new_x,new_y)
    }

    pub fn update_coordinate(&mut self, i : usize, v: Vec<f32>) { //Index,Coordiante(x,y,z)
        self.players[i].x = v[0];
        self.players[i].y = v[1];
        self.players[i].z = v[2];
        self.players_on_map[i].x = ((v[0] - SIZE / 2. + 1.) / SIZE) as i32;
        self.players_on_map[i].y = ((v[1] - SIZE / 2. + 1.) / SIZE) as i32;
        self.players_on_map[i].z = ((v[2] - SIZE / 2. + 1.) / SIZE) as i32;
    }

    pub fn move_pacman(&mut self) {
        let tmp = self.pacman.clone(); //For "prev_place"
        self.update_target(); 

        let is_infer = self.infer_points.iter().any(|x| *x == self.pacman);
        if is_infer {
            let next_point = self.infer_next_point();
            self.pacman = next_point.clone();
            println!("Infer result {}",next_point);
        } else {
            self.pacman = self.routed_next_point();
        }
        self.prev_place = tmp;

        self.move_finalize();
    }

    pub fn move_finalize(&mut self) {
        let mut point_ref = self.field_access_mut_at(&self.pacman.clone()); 
        if *point_ref == 5 { //Bait
            *point_ref = 0;
            self.pacman_score += 1;
            println!("PAC!");
        } else if *point_ref == 6 { //Power bait

            
       }
    }

    pub fn field_access(&self, x : usize,y : usize) -> i32 {
        self.field[x][self.height - y - 1]
    }
    pub fn field_access_at(&self,at : &PlayerOnMap) -> i32 {
        self.field_access(at.x as usize,at.y as usize)
    }
    pub fn field_access_mut(&mut self,x : usize,y : usize) -> &mut i32 {
        &mut self.field[x][self.height - y - 1]
    }
    pub fn field_access_mut_at(&mut self,at : &PlayerOnMap) -> &mut i32 {
        self.field_access_mut(at.x as usize,at.y as usize)
    }
}

pub struct Player {
    pub x : f32,
    pub y : f32,
    pub z : f32,
}

impl Player {
    pub fn new(x : f32,y : f32,z : f32) -> Player{
        Player{x:x,y:y,z:z}
    }
    pub fn coordinate_to_json(&self) -> String {
        let mut ret = String::new();
        ret += "{";
        ret += &format!(r#""x":"{}","y":"{}","z":"{}""#,self.x,self.y,self.z);
        ret += "}";
        ret
    }
    pub fn on_map_coordinate(&self) -> PlayerOnMap { //To integer
        PlayerOnMap::new(
            (self.x / SIZE).ceil() as i32,
            (self.y / SIZE).ceil() as i32,
            (self.z / SIZE) as i32
        )
    }
}

pub struct PlayerOnMap {
    pub x : i32,
    pub y : i32,
    pub z : i32,
}

//const TEST_MAP_DATA : &'static str = "0000\n1111\n2222\n3333\n";
/*
pub fn test_map_creater() -> Map {
    Map::create_by_map_string(TEST_MAP_DATA.to_string())
}
*/

const TEST_MAP_PATH : &'static str = "C:\\Users\\Rock0x3FA\\OtobokeServer\\maps\\test";

#[test]
pub fn map_parse_test() {
    let result = fs::read_to_string(TEST_MAP_PATH.to_string());
    let map_data = result.unwrap();
    //let map_data = TEST_MAP_DATA.to_string();

    let row : Vec<(usize,String)> = 
        map_data.split('\n').
        map(|s| s.to_string()).enumerate().collect();

    let hei = row.len() - 1;
    let wid = row[0].1.len() - 1;

    assert_eq!(hei,4);
    assert_eq!(wid,4);
}

#[test]
pub fn map_access_test() {
    let map = Map::create_by_filename(TEST_MAP_PATH.to_string());
    assert_eq!(map.field_access(0,0),3);
    assert_eq!(map.field_access(3,3),0);
    assert_eq!(map.field_access(1,3),0);
}

#[test]
pub fn map_access_test2() {
    let mut map = Map::create_by_filename(TEST_MAP_PATH.to_string());
    assert_eq!(map.field_access(0,0),3);
    *map.field_access_mut(0,0) = 4;
    assert_eq!(map.field_access(0,0),4);
}



