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
    mpsc::{
        SyncSender,
    },
};

use crate::CommunicationProviderTrait;

impl MapProcAsGame { // for AI
    /// 引数にMapInfoを取り、初期値やフィールドの初期化を行う
    pub fn new(map: MapInfo, snd: SyncSender<String>) -> Self {
        Self {
            pm_inferpoints: map.get_inferpoints(),
            map: map,
            players: vec![],
            //pacman: QuanCoord::default(),
            // dont erase.
            pacman: QuanCoord{ x: 13, y: 13 },
            pm_target: 0,
            pm_state: Arc::new(Mutex::new(super::PMState::Normal)),
            pm_prev_place: QuanCoord{ x: 12, y: 13 },
            comn_prov: None,
            snd: snd,
        }
    }
    fn is_inferpoint_now(&self) -> bool {
        self.pm_inferpoints.exist_in(self.pacman)
    }
    

    /// パワー餌を食べたパックマンを一回動かす関数
    fn move_powered_pacman(&mut self) {
        let movable_next_points = self.map.get_can_move_on(self.pacman);
        let target_player_coord = self.players[self.search_near_player_idx()].coord;

        if !self.is_inferpoint_now() {
            let next = self.routed_next_point(movable_next_points);
            self.move_to(next);
            return;
        }

        // 移動できる方向について移動したあとのプレイヤーとの距離を計算
        let result: Vec<(QuanCoord, f32)> = movable_next_points.iter().
            map(|x| (*x, QuanCoord::dist(target_player_coord, *x))).collect();

        let (mut min_value, mut min_index) = (10000., 0);
        for (i, e) in result.iter().enumerate() {
            if e.1 < min_value {
                min_value = e.1;
                min_index = i;
            }
        }
        self.move_to(result[min_index].0).expect("move to wall");
    }
    /// 通常状態のパックマンを一回動かす関数
    fn move_normal_pacman(&mut self) {
        let movable_next_points = self.map.get_can_move_on(self.pacman);
        if !self.is_inferpoint_now() {
            // not on a infer point.
            let next = self.routed_next_point(movable_next_points);
            self.move_to(next);
            return;
        }

        // TODO: exculude before_change_pos.
        let result: Vec<(QuanCoord, f64)> = movable_next_points.iter().
            map(|x| (*x, self.evaluate_at(*x))).collect();

        // Convert to tuple(coord, score).
        let (mut max_value, mut max_index) = (-10000., 0);
        for (i, e) in result.iter().enumerate() {
            if max_value < e.1 {
                max_value = e.1;
                max_index = i;
            }
        }
        self.move_to(result[max_index].0).expect("move to wall");
    }

    /// パックマンから最も近いプレイヤーのインデックスを返す関数
    fn search_near_player_idx(&self) -> usize {
        let mut near: (f32, usize) = (1000., 0);
        for (i, client) in self.players.iter().enumerate() {
            let player = client.coord;
            let dist = QuanCoord::dist(self.pacman, player);
            if dist < near.0 {
                near = (dist, i);
            }
        }
        near.1
    }

    // Easy AI
    /// パックマンを一回動かす関数
    pub fn move_pacman(&mut self) {
        let powered = match *self.pm_state.lock().unwrap() { // to get &mut self.
            PMState::Powered(_) => true,
            PMState::Normal => false,
        };
        if powered {
            self.move_powered_pacman();
        } else {
            self.move_normal_pacman();

            if self.players.iter().any(|x| QuanCoord::dist(x.coord, self.pacman) <= 1.0) { // pacman game over
                self.snd.send("Pacman died!".to_string());
                self.comn_prov.as_ref().unwrap().
                    send_data_with_tag_and_string("GAMSTA", "PACMAN died".to_string()).unwrap();
                self.snd.send("exit".to_string());
            }



        }
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

                self.handle_powered_cookie();
                
                coord
            },
            _ => {
                coord
            },
        };
        self.pm_prev_place = prev_place_tmp;
        Ok(self.pacman)
    }
    fn cancel_prev_cookie(&self) -> bool { // ignore if pacman is not powered.
        match self.pm_state.lock().unwrap().clone() {
            PMState::Normal => { false },
            PMState::Powered(sender) => {
                // if pacman had already powered.
                sender.send(Message::Stop);
                true
            }
        }
    }
    fn handle_powered_cookie(&mut self) {
        self.cancel_prev_cookie();

        // make time duration
        let d = Duration::from_secs_f64(PACMAN_POWERED_TIME);
        let state_ptr_clone = self.pm_state.clone();
        
        self.snd.send("POWERED!!".to_string());
        Self::pacman_state_change_notify(self.comn_prov.clone().unwrap(), self.pm_state.clone());

        let cloned_prov = self.comn_prov.clone();
        let cloned_sender = self.snd.clone();
        let sender = time_task_reservation(move || {
            *state_ptr_clone.lock().unwrap() = PMState::Normal;
            Self::pacman_state_change_notify(cloned_prov.clone().unwrap(), state_ptr_clone.clone());
            cloned_sender.send("NORMALIZE!!".to_string());
        }, d);

        *self.pm_state.lock().unwrap() = PMState::Powered(sender);
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
    /// 一本道を進む関数。交差点で呼ばれたらpanicする(2つの意味で)
    pub fn routed_next_point(&self, movable_points: Vec<QuanCoord>) -> QuanCoord {
        let next_point: Vec<&QuanCoord> = movable_points.iter().filter(|x| **x != self.pm_prev_place).collect();
        if next_point.len() != 1 {
            panic!("'MapProcAsGame::routed_next_point' must be called in non infer point");
        }
        
        **next_point.first().unwrap()
        
    }
    /// posでの移動評価関数
    fn evaluate_at(&mut self, pos: QuanCoord) -> f64 {
        //see, "パックマンの動き" on https://hackmd.io/VP2HVfw-Rc2COcPSKJQymQ?both 
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
    /// 餌に対して、距離からスコアを算出する 
    fn map_element_bias_with_dist(dist: f64) -> f64 {
        // y = e^(-x). and x >= 0
        (-dist).exp() * 1.0
    }
    /// プレイヤーに対して、距離からスコアを算出する 
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
