#![allow(unused)]
#![allow(dead_code)]

use std::{
    net::{
        TcpStream,
    },
    io::{
        Write,
        BufRead,
        BufReader,
        BufWriter,
    },
    sync::{
        Arc,
        Mutex,
        MutexGuard,
    },
    thread,
};

use super::{
    GameController,
};
use crate::{
    serde::{
        Serialize,
    },
    json::{
        json_build,
        json_build_vec,
    }
};

/// 各クライアントとのBufReader/BufWriterとBuffer(String)を扱う構造体
pub struct CommunicationProvider {
    clients_reader: Vec<Arc<Mutex<BufReader<TcpStream>>>>, 
    clients_writer: Vec<Arc<Mutex<BufWriter<TcpStream>>>>, 
    network_buffer : Vec<Arc<Mutex<String>>>,
}

type ProviderResult = Result<usize, std::io::Error>;
// usize is send length. error is io error.

// TODO: Result<usize, (usize, std::io::Error)>
// Error has client_id is better.

pub trait CommunicationProviderTrait {
    // require methot.
    fn send_bytes(&self, msg: &[u8]) -> ProviderResult;
    fn read_stream_buffer_at(&self, index: usize) -> String;
    fn set_buffer_at(&self, index: usize, s: String);
    /// クローンを取得できる
    fn get_buffer_at(&self, index: usize) -> String;
   
    // provide methot.
    /// 全てのクライアントにmsg(String)を送信し、送ったバイト数を返す
    fn send(&self, msg: String) -> ProviderResult {
        let msg_byte = &msg.into_bytes();
        let ret = self.send_bytes(msg_byte);
        ret
    }
    /// 全てのクライアントにタグ・名前・T型のオブジェクトをJsonとして送信し、送ったバイト数を返す
    fn send_data_with_tag_and_data<T: Serialize>(&self, tag: &str, name: &str, obj: &T) -> ProviderResult {
        self.send(
            json_build(tag, name, obj) + "|"
            )
    }
    /// 全てのクライアントにタグ・名前・Vec<T>型のオブジェクトをJsonとして送信し、送ったバイト数を返す
    fn send_data_with_tag_and_vec_data<T: Serialize>(&self, tag: &str, name: &str, obj: &Vec<T>) -> ProviderResult {
        self.send(
            json_build_vec(tag, name, obj) + "|"
            )
    }
    /// 全てのクライアントに tag;data の形式でデータを送信し、送ったバイト数を返す
    fn send_data_with_tag_and_string(&self, tag: &str, data: String) -> ProviderResult {
        let mut sd = String::new();
        sd += tag;
        sd += ";";
        sd += &data;
        sd += "|";
        self.send(sd)
    }
}

impl CommunicationProviderTrait for CommunicationProvider {
    /// 全てのクライアントにmsg([u8])を送信し、送ったバイト数を返す
    fn send_bytes(&self, msg: &[u8]) -> ProviderResult {
        for client_buf_writer in &self.clients_writer { 
            match client_buf_writer.clone().lock().unwrap().write(msg) {
                Ok(_) => { /* */ },
                Err(r) => { return Err(r); }
            }
            client_buf_writer.clone().lock().unwrap().flush();
        }
        Ok(msg.len())
    }
    fn read_stream_buffer_at(&self, index: usize) -> String {
        read_by_buffer(self.clients_reader[index].clone())
    }
    fn set_buffer_at(&self, index: usize, s: String) {
        *(self.network_buffer.clone()[index].lock().unwrap()) = s;
    }
    /// クローンを取得できる
    fn get_buffer_at(&self, index: usize) -> String {
        self.network_buffer.clone()[index].lock().unwrap().clone()
    }
}

impl CommunicationProvider {
    pub fn new() -> Self {
        Self {
            clients_reader: vec![],
            clients_writer: vec![],
            network_buffer: vec![],
        }
    }
    /// クライアント数を返す
    pub fn clients_count(&self) -> usize {
        assert_eq!(self.clients_reader.len(), self.clients_writer.len());
        self.clients_reader.len()
    }
    /// CommunicationProviderの各要素を新しいクライアント用にそれぞれ追加する
    pub fn push_client(&mut self, stream: TcpStream, def_str: String) {
        self.clients_reader.push(Arc::new(Mutex::new(BufReader::new(stream.try_clone().unwrap()))));
        self.clients_writer.push(Arc::new(Mutex::new(BufWriter::new(stream.try_clone().unwrap()))));
        self.network_buffer.push(Arc::new(Mutex::new(def_str)));
    }
    /// indexで指定したクライアントのBufReaderを取得
    fn get_buf_reader_at(&self, index: usize) -> Arc<Mutex<BufReader<TcpStream>>> {
        self.clients_reader[index].clone()
    }
    /// indexで指定したクライアントのBufWriterを取得
    fn get_buf_writer_at(&self, index: usize) -> Arc<Mutex<BufWriter<TcpStream>>> {
        self.clients_writer[index].clone()
    }
}

impl CommunicationProviderTrait for Arc<Mutex<CommunicationProvider>> {
    fn send_bytes(&self, msg: &[u8]) -> ProviderResult {
        self.lock().unwrap().send_bytes(msg)
    }
    fn read_stream_buffer_at(&self, index: usize) -> String {
        self.lock().unwrap().read_stream_buffer_at(index)
    }
    fn set_buffer_at(&self, index: usize, s: String) {
        self.lock().unwrap().set_buffer_at(index, s);
    }
    /// クローンを取得できる
    fn get_buffer_at(&self, index: usize) -> String {
        self.lock().unwrap().get_buffer_at(index)
    }
}



impl GameController {
    /// 全てのクライアントにmsg(String)を送信し、送ったバイト数を返す
    pub fn announce_wrap(&self, msg: String){
        self.comn_prov.send(msg).expect("Could not send data");
    }
    /// msg(String)を','で切り分け、Vec<f32>で返す
    pub(super) fn parse_client_info(msg: String) -> Vec<f32> {
        msg.trim().split(',').
            map(|e| e.parse().
                expect("Could not parse data")).
                collect()
    }
    /// 各プレイヤーの座標の読み取りを開始
    pub(super) fn start_reading_coordinate(&self) {
        let player_count = self.comn_prov.lock().unwrap().clients_count();

        // if data received, a mutex variable self.network_buffer change.
        for i in 0..player_count {
            let i_th_bufstr = self.comn_prov.lock().unwrap().get_buf_reader_at(i);
            let dest_of_write_buffer_sem = self.comn_prov.clone();
            thread::spawn(move || {
                loop {
                    let mut client_data = String::new();
                    i_th_bufstr.clone().lock().unwrap().read_line(&mut client_data).unwrap();
                    dest_of_write_buffer_sem.set_buffer_at(i, client_data);
                }
            });
        }
    }
}

/// バッファからデータを読み取りStringを返す
pub fn read_by_buffer(bs: Arc<Mutex<BufReader<TcpStream>>>) -> String {
    let mut ret = String::new();
    bs.clone().lock().unwrap().read_line(&mut ret).unwrap();
    ret
}
