use std::fs;

pub struct Player {
    pub x : f32,
    pub y : f32,
    pub z : f32,
}

pub struct Map {
    pub width : usize,
    pub height :usize,
    pub field : Vec<Vec<i32>>,
    pub players : Vec<Player>,
    pub packman : Player,
}

impl Map {
    pub fn new(w : usize ,h : usize) -> Map {
        Map{width:w,height:h,field:vec![vec![0;h];w],players:vec![],packman:Player::new(3.,3.,3.)}
    }

    pub fn create_by_filename(file_name : String) -> Map {
        let result = fs::read_to_string(&file_name);
        let map_data : String;
        match result   {
            Ok(string) =>  {
                map_data = string;
            }
            Err(_) => {
                panic!("[Error]File {} is not exist.",&file_name);
            }
        }
        
        let row : Vec<(usize,String)> = map_data.split('\n').map(|s| s.to_string()).enumerate().collect();
        //dont handle all elements length is not equal. mendo-kusai

        let hei = row.len();
        let wid = row[0].1.len();

        println!("{},{}",hei,wid);

        let mut map_tmp : Vec<Vec<i32>> = vec![vec![0;hei];wid];
        for (i,element) in row {
            //&element[0..(5)].to_string().chars().enumerate().for_each(|(j,nest)| map_tmp[j][i] = nest as i32 - 48);
            element.chars().enumerate().for_each(|(j,nest)| {
                if j < wid  { map_tmp[j][i] = nest as i32 - 48; } 
            });
        }

        Map{width:wid - 1,height:hei,field:map_tmp,players:vec![],packman:Player::new(3.,3.,3.)}
    }
    
    pub fn show_map(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                print!("{}",self.field[x][y]);
            }
            println!("");
        }
    }

    pub fn map_to_string(&self) -> String {
        let mut ret = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                ret += &self.field[x][y].to_string();
            }
            ret += ",";
        }
        ret + "|"
    }
    pub fn coordinate_to_json(&self) -> String {
        let mut ret = String::new();
        ret += r#"{"Coordinate":["#;
        for i in 0..self.players.len() - 1 {
            ret += &(self.players[i].coordinate_to_json() + ",");
        }
        ret += &(self.players[self.players.len() - 1].coordinate_to_json());
        ret += r#"],"Packman":"#;
        ret += &self.packman.coordinate_to_json();
        ret += "}|";
        ret
    }
}

impl Player {
    pub fn new(x : f32,y : f32,z : f32) -> Player{
        Player{x:x,y:y,z:z}
    }
    pub fn coordinate_to_json(&self) -> String {
        let mut ret = String::new();
        ret += "{";
        ret += &format!(r#""x":"{}","y":"{}","z":"{}""#,self.x,self.y,self.z);
        ret += "}";
        ret
    }
}
