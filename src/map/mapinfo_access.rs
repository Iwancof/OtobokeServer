

use super::{
    MapInfo,
    QuanCoord,
};

impl MapInfo {
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

}
