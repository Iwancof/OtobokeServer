use super::{
    time,
};

use std::{
    sync::{
        mpsc::{
            Sender,
        },
        Arc,
        Mutex,

    },
    collections::{
        HashMap,
    },
};

pub mod mapinfo_access;
pub mod mapinfo_utils;
pub mod coord;
pub mod map_alg;
pub mod test;

use coord::{
    QuanCoord,
    RawCoord,
};

use crate::{
    CommunicationProviderTrait,
    CommunicationProvider,
};

/// Unityの実装に合わせる。ブロックのサイズ
pub const UNIT_SIZE: f32 = 1.05;
/// 単一であることが保証されているマップ情報に対して高速検索を行うための要素(3, 4はテレポート)
pub const UNIQUE_ELEMENTS: [i32; 2] = [3, 4];
/// パックマンが無敵状態でいることのできる時間
pub const PACMAN_POWERED_TIME: f64 = 8.0; // 8 sec


/// 純粋なマップ情報
pub struct MapInfo {
    pub width: usize,
    pub height: usize,
    pub field: Vec<Vec<i32>>,
    pub unique_points: HashMap<i32, QuanCoord>,
}

/// パックマンの動作や、クライアントへの通知を含めたマップ管理構造体
pub struct MapProcAsGame {
    pub map: MapInfo,
    pub players: Vec<GameClient>, 
    pub pacman: QuanCoord,
    pub pm_target: usize,
    pub pm_inferpoints: Vec<QuanCoord>,
    pub pm_state: Arc<Mutex<PMState>>,
    pub pm_prev_place: QuanCoord,
    pub comn_prov: Option<Arc<Mutex<CommunicationProvider>>>,
    // TODO: fix to use trait.
}

/// 各クライアントの座標。RawCoordを保存しておくことで、他プレイヤーへの送信を高速化している。
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct GameClient {
    /// クライアントの量子化座標
    pub coord: QuanCoord,
    /// クライアントの非量子化座標。計算ではなく、送信されたデータをそのまま保存
    pub raw_coord: RawCoord,
}

/// パックマンの状況
#[derive(Clone)]
pub enum PMState {
    /// 通常状態
    Normal,
    /// パワー状態。SenderにStopシグナルを送信すると、Normalに戻るタイマーを無効にできる
    Powered(Sender<time::Message>), // this sender is to stop thread. 
}

impl ToString for PMState {
    fn to_string(&self) -> String {
        let mut ret = String::new();
        match *self {
            Self::Normal => "Normal".to_string(),
            Self::Powered(_) => "Powered".to_string(),
        }
    }
}









