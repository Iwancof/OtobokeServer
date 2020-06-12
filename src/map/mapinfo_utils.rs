#![warning(unused)]
#![allow(dead_code)]

use std::fs;
use std::sync::{
    Arc, 
    Mutex, 
};
use std::collections::HashMap;

use super::{
    MapInfo,
    MapProcAsGame,
    PMState,
    UNIQUE_ELEMENTS,
    coord::QuanCoord,
};

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
}

