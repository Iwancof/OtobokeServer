
use std::{
    fmt::{
        self,
        Debug,
    },
    ops::{
        Sub,
    },
};

use super::{
    UNIT_SIZE,
};

#[derive(Copy, Clone, Debug, Serialize, PartialEq, Default)]
pub struct QuanCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, Serialize, PartialEq, Default)]
pub struct RawCoord {
    // for use communicate with clients(Unity)
    pub x: f32,
    pub y: f32,
    pub z: f32, // clients(Unity) send z data.
}

impl RawCoord {
    pub fn quantize(&self) -> QuanCoord {
        QuanCoord {
            x: ((self.x - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
            y: ((self.y - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
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
    pub fn torus_form(&self, map_ptr: &super::MapInfo) -> Self {
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

