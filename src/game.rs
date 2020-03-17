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
        }
    }

    pub fn create_by_filename(file_name : String) -> Map {
        let result = fs::read_to_string(&file_name);
        let map_data : String;
        match result   {
            Ok(string) =>  {
                map_data = string;
            }
            Err(_) => {
                panic!("[Error]File {} is not exist.",&file_name);
            }
        }
        
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
        }
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

    pub fn update_coordinate(&mut self, i : usize, v: Vec<f32>) { //Index,Coordiante(x,y,z)
        self.players[i].x = v[0];
        self.players[i].y = v[1];
        self.players[i].z = v[2];
        self.players_on_map[i].x = ((v[0] - SIZE / 2. + 1.) / SIZE) as i32;
        self.players_on_map[i].y = ((v[1] - SIZE / 2. + 1.) / SIZE) as i32;
        self.players_on_map[i].z = ((v[2] - SIZE / 2. + 1.) / SIZE) as i32;
    }

    pub fn move_pacman(&mut self) {
        //println!("Function called");


        let tmp = self.pacman.clone();
        //println!("Here {}",self.pacman);

        self.update_target(); 
        //println!("{}",self.infer_points.len());
        //println!("{}",self.pacman);
        let is_infer = self.infer_points.iter().any(|x| *x == self.pacman);
        if is_infer {
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
            self.pacman = can_moves[0].clone();
            println!("Infer result {}",can_moves[0]);
        } else {
            let (mut new_x,mut new_y) = (0,0);
            for dx in vec![-1,1] {
                let (x,y) : (i32,i32) = (
                    (self.pacman.x as i32 + dx) % self.width as i32,
                    self.pacman.y as i32
                    );
                if self.can_move(x,y) {
                    new_x = x;
                    new_y = y;
                }
            }
            for dy in vec![-1,1] {
                let (x,y) : (i32,i32) = (
                    self.pacman.x as i32,
                    (self.pacman.y as i32 + dy) % self.height as i32
                    );
                if self.can_move(x,y) {
                    new_x = x;
                    new_y = y;
                }
            }

            self.pacman.x = new_x;
            self.pacman.y = new_y
        }
        self.prev_place = tmp;

        //println!("Function exit");
    }

    pub fn field_access(&self, x : usize,y : usize) -> i32 {
        self.field[x][self.height - y - 1]
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

#[test]
pub fn map_parse_test() {
    let result = fs::read_to_string("C:\\Users\\Rock0x3FA\\OtobokeServer\\maps\\test");
    let map_data = result.unwrap();
    
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
    let map = Map::create_by_filename("C:\\Users\\Rock0x3FA\\OtobokeServer\\maps\\test".to_string());
    assert_eq!(map.field_access(0,0),3);
    assert_eq!(map.field_access(3,3),0);
    assert_eq!(map.field_access(1,3),0);

}
