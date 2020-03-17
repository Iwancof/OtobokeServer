#![allow(unused)]

use std::fs;
use std::fmt;
use super::game::{Map,PlayerOnMap,Player};
use std::ops::Sub;

impl Map {
    pub fn update_target(&mut self) -> usize{ //Change this methot if you want to change targets
        let (mut dist,mut index) : (f32,usize) = (10000.,4); 
        for (i,e) in self.players_on_map.iter().enumerate() {
            let d = e.dist(&self.pacman);
            if d < dist {
                dist = d;
                index = i;
            }
        };
        self.target_index = index;
        index
    }

    pub fn find_infer_point(&mut self) -> Vec<PlayerOnMap> {
        let mut ret = vec![];
        for x in 0..self.width as i32{
            for y in 0..self.height as i32 {
                let num = self.count_on(x,y);
                if self.field[x as usize][y as usize] != 1 && 3 <= num { //Inter point
                    ret.push(PlayerOnMap::new2(x,y));
                }
            }
        }
        ret
    }
    pub fn count_on(&self,x : i32 ,y : i32) -> i32 {
        let mut ret = 0;
        for dx in vec![-1,1] {
            let (x,y) = (x + dx,y);
            if x < 0 || y < 0 || self.width as i32 <= x || self.height as i32 <= y {
                continue;
            }
            if self.field[x as usize][y as usize] != 1 {
                ret += 1;
            }
        }
        for dy in vec![-1,1] {
            let (x,y) = (x,y + dy);
            if x < 0 || y < 0 || self.width as i32 <= x || self.height as i32 <= y {
                continue;
            }
            if self.field[x as usize][y as usize] != 1 {
                ret += 1;
            }
        }
        ret
    }

    pub fn convert_system(&self,arg : Vec<PlayerOnMap>) -> Vec<PlayerOnMap> {
        arg.iter().map(|x| x.convert_system(self.height as i32)).collect()
    }

    pub fn can_move(&self,x : i32,y : i32) -> bool {
        //let (x,y) : (i32,i32) = (self.pacman.x as i32 + dx,self.pacman.y as i32);
        if x < 0 || y < 0 || self.width as i32 <= x || self.height as i32 <= y {
            return false;
        }
        if self.field_access(x as usize,y as usize) == 1 {
            return false;
        }
        if self.prev_place.x == x && self.prev_place.y == y {
            return false;
        }
        true
    }
}


pub struct Set {
    pub x : i32,
    pub y : i32,
}

impl Set {
    pub fn new(x:i32,y:i32) -> Set {
        Set{x:x,y:y}
    }
}


impl PlayerOnMap {
    pub fn new(x : i32,y : i32,z : i32) -> PlayerOnMap {
        PlayerOnMap{x:x,y:y,z:z}
    }
    pub fn new2(x : i32,y : i32) -> PlayerOnMap {
        PlayerOnMap{x:x,y:y,z:0}
    }
    pub fn convert_system(&self,height : i32) -> PlayerOnMap {
        PlayerOnMap::new2(self.x,height - self.y - 1)
    }

    pub fn coordinate_to_json(&self) -> String {
        let mut ret = String::new();
        ret += "{";
        ret += &format!(r#""x":"{}","y":"{}","z":"{}""#,self.x,self.y + 1,self.z);
        ret += "}";
        ret

    }
    pub fn dist(&self,t : &PlayerOnMap) -> f32 {
        (
            (self.x as f32 - t.x as f32).powf(2.) + 
            (self.y as f32 - t.y as f32).powf(2.)
        ).sqrt()
    }
    pub fn to_set(&self) -> Set {
        Set {
            x : self.x,
            y : self.y
        }
    }
}

impl Sub for PlayerOnMap {
    type Output = PlayerOnMap;
    fn sub(self,other : PlayerOnMap) -> PlayerOnMap {
        PlayerOnMap::new2(
            self.x - other.x,
            self.y - other.y
        )
    }
}

impl fmt::Display for PlayerOnMap {
    fn fmt(&self,f : &mut fmt::Formatter) -> fmt::Result {
        write!(f,"({},{},{})",self.x,self.y,self.z)
    }
}

impl fmt::Display for Set {
    fn fmt(&self,f : &mut fmt::Formatter) -> fmt::Result {
        write!(f,"({},{})",self.x,self.y)
    }
}

impl PartialEq for Set {
    fn eq(&self, other : &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl PartialEq for PlayerOnMap {
    fn eq(&self, other : &Self) -> bool {
        self.x == other.x &&
        self.y == other.y
        //Ignore z because this game is 2d
    }
}

impl Clone for PlayerOnMap {
    fn clone(&self) -> PlayerOnMap {
        PlayerOnMap{x:self.x,y:self.y,z:self.z}
    }
}

#[test]
pub fn count_test() {
    let mut m = Map::new(4,4);
    m.field[1][0] = 1;
    m.field[1][2] = 1;
    m.field[0][1] = 1;
    //0000
    //0100
    //1000
    //0100

    //Point counter check
    assert_eq!(m.count_on(0,0),0);
    assert_eq!(m.count_on(1,1),1);
    assert_eq!(m.count_on(2,2),3);
}

#[test]
pub fn infer_point_test() {
    let mut m = Map::new(4,4);
    m.field[1][0] = 1;
    m.field[1][2] = 1;
    m.field[0][1] = 1;
    //0000
    //0100
    //1000
    //0100

    //Infer point vector check
    let ret = m.find_infer_point();
    let expected_result = vec![
        PlayerOnMap::new2(2,1),
        PlayerOnMap::new2(3,1),
        PlayerOnMap::new2(2,2),
        PlayerOnMap::new2(3,2),
        PlayerOnMap::new2(2,3),
    ];

    assert_eq!(ret.len(),expected_result.len());
    for e in ret {
        assert!(expected_result.iter().any(|x| *x == e));
    }
}

#[test]
pub fn convert_system_test() {
    let x = PlayerOnMap::new2(3,3);
    let converted = x.convert_system(5);
    
    assert!(converted == PlayerOnMap::new2(3,1));
}
