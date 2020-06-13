#![allow(unused)]

use std::fs;
use std::fmt;
use super::{
    PACMAN_POWERED_TIME,
    MapProcAsGame,
    MapInfo,
    PMState,
    coord::{
        QuanCoord,
    },
};
use super::{
    time::{
        time_task_reservation,
        Message,
    },
};
use std::ops::Sub;
use std::cmp::Ord;
use std::time::{
    Duration,
};
use std::sync::{
    Arc,
    Mutex,
};

use crate::CommunicationProviderTrait;

/// パックマンの動作や、クライアントへの通知を含めたマップ管理構造体
impl MapProcAsGame { // for AI
    /// 引数にMapInfoを取り、初期値やフィールドの初期化を行う
    pub fn new(map: MapInfo) -> Self {
        Self {
            pm_inferpoints: map.get_inferpoints(),
            map: map,
            players: vec![],
            //pacman: QuanCoord::default(),
            // dont erase.
            pacman: QuanCoord{ x: 24, y: 16 },
            pm_target: 0,
            pm_state: Arc::new(Mutex::new(super::PMState::Normal)),
            pm_prev_place: QuanCoord{ x: 25, y: 16 },
            comn_prov: None,
        }
    }

    // Easy AI
    /// パックマンを一回動かす関数
    pub fn move_pacman(&mut self) {
        let movable_next_points = self.map.get_can_move_on(self.pacman);
        if !self.pm_inferpoints.exist_in(self.pacman) {
            // not on a infer point.
            let next = self.routed_next_point(movable_next_points);
            self.move_to(next);
            return;
        }

        for e in &movable_next_points {
            //println!("able {}", *e);
        }

        // TODO: except before_change_pos.
        let result: Vec<(QuanCoord, f64)>= movable_next_points.iter().
            map(|x| (*x, self.evaluate_at(*x))).collect();
            // Convert to tuple(coord, score).
        let (mut max_value, mut max_index) = (-10000., 0);
        for (i, e) in result.iter().enumerate() {
            if max_value < e.1 {
                max_value = e.1;
                max_index = i;
            }
        }
        //println!("result is {:?}", result[max_index]);
        self.move_to(result[max_index].0).expect("move to wall");
    }
    /// パックマンを指定の座標まで一回で移動させる。テレポートなども考慮される
    pub fn move_to(&mut self, coord: QuanCoord) -> Result<QuanCoord, QuanCoord>{
        let mut prev_place_tmp = self.pacman; // if not error occured. this will be pm_prev_place
        self.pacman = match self.map.access_by_coord_game_based_system(coord) {
            1 => {
                // move to wall.
                return Err(coord);
            },
            3 => {
                prev_place_tmp = coord;
                *self.map.unique_points.get(&4).unwrap()
            },
            4 => {
                prev_place_tmp = coord;
                *self.map.unique_points.get(&3).unwrap()
            },
            5 => { // normal cookie
                self.pac_cookie_at(coord);
                *self.map.access_by_coord_game_based_system_mutref(coord) = 0; // pac.
                coord
            },
            6 => { // power cookie
                self.pac_cookie_at(coord);
                *self.map.access_by_coord_game_based_system_mutref(coord) = 0; // pac.
           
                match self.pm_state.lock().unwrap().clone() {
                    PMState::Normal => { /* nothing */ },
                    PMState::Powered(sender) => {
                        // if pacman had already powered.
                        sender.send(Message::Stop);
                    }
                }

                // make time duration
                let d = Duration::from_secs_f64(PACMAN_POWERED_TIME);
                let state_ptr_clone = self.pm_state.clone();
                
                println!("POWERED!!");
                Self::pacman_state_change_notify(self.comn_prov.clone().unwrap(), self.pm_state.clone());

                let cloned_prov = self.comn_prov.clone();
                let sender = time_task_reservation(move || {
                    *state_ptr_clone.lock().unwrap() = PMState::Normal;
                    Self::pacman_state_change_notify(cloned_prov.clone().unwrap(), state_ptr_clone.clone());
                    println!("Normalize");
                }, d);

                *self.pm_state.lock().unwrap() = PMState::Powered(sender);
                
                coord
            },
            _ => {
                coord
            },
        };
        self.pm_prev_place = prev_place_tmp;
        Ok(self.pacman)
    }
    fn pac_cookie_at(&mut self, coord: QuanCoord) {
        match self.map.access_by_coord_game_based_system(coord) {
            5 | 6 => {
                self.comn_prov.as_ref().unwrap().send_data_with_tag_and_data("PACCOL", "Coordinate", &coord).unwrap();
            },
            _ => {
                panic!("pac cookie got invalid coordinate");
            }
        };
    }
    fn pacman_state_change_notify<T: CommunicationProviderTrait>(prov: T, state: Arc<Mutex<PMState>>) {
        &prov.send_data_with_tag_and_string("PACSTA", state.lock().unwrap().to_string()).unwrap();
    }
    pub fn routed_next_point(&self, movable_points: Vec<QuanCoord>) -> QuanCoord {
        let next_point: Vec<&QuanCoord> = movable_points.iter().filter(|x| **x != self.pm_prev_place).collect();
        if next_point.len() != 1 {
            panic!("'MapProcAsGame::routed_next_point' must be called in non infer point");
        }
        
        **next_point.first().unwrap()
        
    }
    fn evaluate_at(&mut self, pos: QuanCoord) -> f64 {
        //see, "パックマンの動き" om https://hackmd.io/VP2HVfw-Rc2COcPSKJQymQ?both 
        let mut attractive_score: f64 = 0.;
        for y in 0..self.map.height {
            for x in 0..self.map.width {
                if !((pos.x - x as i32).abs() < 10 && (pos.y - y as i32).abs() < 10) {
                    continue;
                }
                let dist = pos.distance_to_element(x as i32, y as i32);
                attractive_score += match self.map.access_by_coord_game_based_system(QuanCoord{x: x as i32, y: y as i32}) {
                    // calc map bias
                    0 => 0., // no data
                    1 => 0., // wall
                    2 => 0., // start point
                    3 => 0., // teleport1
                    4 => 0., // teleport2
                    
                    5 => 1., // notmal bait
                    6 => 50.,// power bait
                    _ => 0.,
                    // TODO: if pacman state is invers.
                } * Self::map_element_bias_with_dist(dist);
            }
        }
        let mut pl_score = 0.;
        for p in &self.players {
            let dist = pos.distance_to_coord(p.coord);
            pl_score += 1. * Self::player_pos_bias_with_dist(dist);
            // '1' is players power.
            // if the player is strong,
            // escaping from the player is 
            // high priority.
        }
        //println!("{} bait: {}, player: {}", pos, attractive_score, pl_score);
        attractive_score += pl_score;
        //println!("evalued at {:?}, score is {}", pos, attractive_score);
        attractive_score
    }
    fn map_element_bias_with_dist(dist: f64) -> f64 {
        // y = e^(-x). and x >= 0
        (-dist).exp() * 1.0
    }
    fn player_pos_bias_with_dist(dist: f64) -> f64 {
        -100. / (dist.powf(4.))
    }
}


trait ExistVecTrait<T> {
    fn exist_in(&self, e: T) -> bool;
}

impl<T: PartialEq> ExistVecTrait<T> for Vec<T> {
    fn exist_in(&self, e: T) -> bool {
        self.iter().any(|x| *x == e)
    }
}

#[test]
fn exist_vec_trait_test() {
    let v = vec![1, 2, 4, 5];
    assert_eq!(v.exist_in(1), true);
    assert_eq!(v.exist_in(2), true);
    assert_eq!(v.exist_in(3), false);
    assert_eq!(v.exist_in(4), true);
    assert_eq!(v.exist_in(5), true);
    assert_eq!(v.exist_in(6), false);
}
