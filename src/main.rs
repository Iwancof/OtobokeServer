#![allow(unused)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

mod server;
mod game;
mod time;
mod map;
mod json;
mod system_observer;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::env;
use map::MapInfo;
use std::thread;
use std::cmp::max;
use std::process::exit;
use std::sync::{
    Arc,
    Mutex,
    mpsc::{
        Sender,
        Receiver,
        sync_channel,
    }
};
use std::time::{
    Duration,
};
use server::{
    communication::{
        CommunicationProviderTrait,
        CommunicationProvider,
    },
};
use getch::Getch;

fn main() {
    clear();

    println!("OTOBOKE SERVER STARTING NOW!!");

    let (snd, rcv) = sync_channel(16);

    let args: Vec<String> = env::args().collect();
    let mut map_path = args[0].trim_matches(|c| c != '\\').to_string();
    map_path += "..\\..\\maps\\second_map";

    println!("Use default map. {}",map_path);

    let player_number = 1;

    let mut map = map::MapInfo::build_by_filename(map_path);
    // pure map info

    let mut game = game::Game::new(map, player_number, snd.clone());
    // make instance with map and number.

    let mut g = server::GameController::new(game, snd.clone());
    // make server call GameController

    thread::spawn(move || {
        g.start_server();
    });

    let mut logs: Vec<String> = vec![];
    let writing_logs = Arc::new(Mutex::new(false));

    let c_wric = writing_logs.clone();
    thread::spawn(move || {
        let mut tmp_chars = String::new();
        let getch = Getch::new();
        loop {
            if let Ok(c) = getch.getch() {
                match c {
                    8 => { tmp_chars.pop(); },
                    13 => {
                        if tmp_chars == "exit".to_string() {
                            exit(0);
                        } else if tmp_chars == "".to_string() {
                            ;
                        } else {
                            // unknown command.
                            snd.send(format!("{} is not a command", tmp_chars));
                        }
                        tmp_chars = String::new();
                    },
                    c => {
                        match c as char {
                            c => tmp_chars.push(c),
                        }
                        if *c_wric.lock().unwrap() {
                            continue;
                        }
                    }
                };
                println!("\x1b[25;0H\x1b[2K{}", tmp_chars);
            }
        }
    });
    loop {
        if let Ok(l) = rcv.try_recv() {
            if l == "exit".to_string() {
                exit(0);
            }
            logs.push(l);
        }
        *writing_logs.lock().unwrap() = true;
        print!("\x1b[7;0H");
        for j in max(logs.len() as i32 - 10, 0)..logs.len() as i32 {
            println!("{}", logs[j as usize]);
        }
        *writing_logs.lock().unwrap() = false;

        thread::sleep(Duration::from_millis(100));
    }
}

fn print_on(msg: String, wos: usize, hos: usize) {
    print!("\x1b[{};{}H{}", hos, wos, msg);
}

fn clear() {
    println!("\x1b[2J");
    println!("\x1b[0;0H");
}
