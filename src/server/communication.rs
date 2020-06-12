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


pub struct CommunicationProvider {
    pub clients: Vec<Arc<Mutex<BufStream>>>, 
    pub network_buffer : Vec<Arc<Mutex<String>>>,
}

type ProviderResult = Result<usize, std::io::Error>;
// usize is send length. error is io error.

// TODO: Result<usize, (usize, std::io::Error)>
// Error has client_id is better.

pub trait CommunicationProviderTrait {
    // require methot.
    fn send_bytes(&self, msg: &[u8]) -> ProviderResult;
   
    // provide methot.
    fn send(&self, msg: String) -> ProviderResult {
        println!("Sending... {}", msg);
        let msg_byte = &msg.into_bytes();
        self.send_bytes(msg_byte)
    }
    fn send_data_with_tag_and_data<T: Serialize>(&self, tag: &str, name: &str, obj: &T) -> ProviderResult {
        self.send(
            json_build(tag, name, obj) + "|"
            )
    }
    fn send_data_with_tag_and_vec_data<T: Serialize>(&self, tag: &str, name: &str, obj: &Vec<T>) -> ProviderResult {
        self.send(
            json_build_vec(tag, name, obj) + "|"
            )
    }
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
    fn send_bytes(&self, msg: &[u8]) -> ProviderResult {
        for client_arc in &self.clients { 
            match client_arc.lock() {
                Ok(mut client) => {
                    match client.write(msg) {
                        Ok(_) => { /* */ },
                        Err(r) => { return Err(r); }
                    }
                },
                Err(_) => {
                    println!("Could not send data");
                }
            }
        }
        Ok(msg.len())
    }
}

impl CommunicationProvider {
    pub fn new() -> Self {
        Self {
            clients: vec![],
            network_buffer: vec![],
        }
    }
    fn clients_count(&self) -> usize {
        self.clients.len()
    }
}

impl CommunicationProviderTrait for Arc<Mutex<CommunicationProvider>> {
    fn send_bytes(&self, msg: &[u8]) -> ProviderResult {
        self.lock().unwrap().send_bytes(msg)
    }
}



impl GameController {
    pub fn announce_wrap(&self, msg: String){
        self.comn_prov.send(msg).expect("Could not send data");
    }
    pub(super) fn parse_client_info(msg: String) -> Vec<f32> {
        msg.split(',').
            map(|e| e.parse().
                expect("Could not parse data")).
                collect()
    }
    pub(super) fn start_reading_coordinate(&self) {
        let stream_cloned: Vec<Arc<Mutex<BufStream>>> = self.comn_prov.lock().unwrap().clients.clone();
        let buffer_cloned: Vec<Arc<Mutex<String>>> = self.comn_prov.lock().unwrap().network_buffer.clone();

        // if data received, a mutex variable self.network_buffer change.
        for (i, stream_arc_non_static) in stream_cloned.iter().enumerate() {
            let stream_arc = stream_arc_non_static.clone(); // create 'static pointer
            let mut data_buffer = buffer_cloned.clone();
            thread::spawn(move || {
                loop {
                    match stream_arc.lock() {
                        Ok(mut stream) => {
                            let mut read_data = String::new();
                            stream.rd.read_line(&mut read_data).unwrap();
                            *(data_buffer[i].lock().unwrap()) = read_data.clone();
                        },
                        Err(_) => {
                            println!("[warn] Could not lock client in start_reading_coordinate");
                        }
                    }
                }
            });
        }
    }
}

pub fn read_by_buffer(bs: Arc<Mutex<BufStream>>) -> String {
    let mut ret = String::new();
    let reader = &mut bs.lock().expect("Could not lock buffer stream").rd;
    reader.read_line(&mut ret).unwrap();
    ret
}


pub struct BufStream {
    pub rd: BufReader<TcpStream>,
    pub wr: BufWriter<TcpStream>,
}

impl BufStream {
    pub fn new(stream: &TcpStream) -> BufStream {
        BufStream{
            rd: BufReader::new(
                stream.try_clone().unwrap()),
            wr: BufWriter::new(
                stream.try_clone().unwrap())}
    }
    pub fn read_string(&mut self) -> String {
        let mut ret = String::new();
        self.rd.read_line(&mut ret).unwrap();
        ret
    }
    pub fn write(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        println!("send");
        self.wr.write(data);
        self.wr.flush()
    }
}
