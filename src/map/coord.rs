
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

/// ゲーム内の整数座標を扱う構造体
/// 量子化された(Quantize)座標(Coordinate)から
#[derive(Copy, Clone, Debug, Serialize, PartialEq, Default)]
pub struct QuanCoord {
    pub x: i32,
    pub y: i32,
}

/// Unityとの通信用に使う座標を扱う構造体
#[derive(Copy, Clone, Debug, Serialize, PartialEq, Default)]
pub struct RawCoord {
    // for use communicate with clients(Unity)
    pub x: f32,
    pub y: f32,
    /// Unityとの互換性を考え実装
    pub z: f32, // clients(Unity) send z data.
}

impl RawCoord {
    /// 量子化
    pub fn quantize(&self) -> QuanCoord {
        QuanCoord {
            x: ((self.x - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
            y: ((self.y - UNIT_SIZE / 2. + 1.) / UNIT_SIZE) as i32,
        }
    }
}

impl QuanCoord {
    /// 反量子化メゾット
    /// zは0に設定される
    pub fn anti_quantize(&self) -> RawCoord {
        RawCoord {
            x: self.x as f32 * UNIT_SIZE,
            y: (self.y as f32 + 1.) * UNIT_SIZE,
            z: 0.,
        }
    }
    /// QuanCoord間の距離
    ///
    /// # Examples
    /// ```
    /// let quan_coord1 = QuanCoord{x: 0, y: 0};
    /// let quan_coord2 = QuanCoord{x: 1, y: 1};
    /// assert_eq!(QuanCoord::dist(quan_coord1, quan_coord2), 2.0.sqrt());
    /// ```
    pub fn dist(a: QuanCoord, b: QuanCoord) -> f32 {
        (
            (a.x as f32 - b.x as f32).powf(2.) +
            (a.y as f32 - b.y as f32).powf(2.)
        ).sqrt()
    }
    /// 与えられたQuanCoordのxフィールドに、引数xを加算したQuanCoordを返す
    /// # Examples
    /// ```
    /// let quan_coord = QuanCoord{x: 1, y: 2};
    /// assert_eq!(quan_coord.plus_element_x(3), QuanCoord{x: 4, y: 2});
    /// ```
    pub fn plus_element_x(&self, x: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }
    /// 与えられたQuanCoordのyフィールドに、引数xを加算したQuanCoordを返す
    /// # Examples
    /// ```
    /// let quan_coord = QuanCoord{x: 1, y: 2};
    /// assert_eq!(quan_coord.plus_element_y(3), QuanCoord{x: 1, y: 5});
    /// ```
    pub fn plus_element_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }
    /// 各フィールドをそれぞれ幅、高さでの余剰を求める。その際、MapInfoへのポインタが必要になる。
    /// # Examples
    /// ```
    /// let quan_coord = QuanCoord{x: -1, y: 5};
    /// assert_eq!(quan_coord.torus_from(map), QuanCoord{x: 2, y: 1});
    /// // map is 3 * 5.
    /// ```
    pub fn torus_form(&self, map_ptr: &super::MapInfo) -> Self {
        Self {
            x: (self.x as i32 + map_ptr.width as i32) % map_ptr.width as i32,
            y: (self.y as i32 + map_ptr.height as i32) % map_ptr.height as i32,
        }
    }
    /// 自身から引数の(x, y)への距離を返す
    pub fn distance_to_element(&self, x: i32, y: i32) -> f64 {
        (((self.x - x) as f64).powf(2.) + ((self.y - y) as f64).powf(2.)).sqrt()
    }
    /// 自身から引数のotherへの距離を返す
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

