
use super::{
    serde_derive,
    serde::{
        self,
        Serialize,
    },
    serde_json,
};


pub fn json_build<T: Serialize>(tag: &str, name: &str, object: &T) -> String {
    let mut ret = String::new();
    ret += &format!("{};", tag);
    ret += "{\"";
    ret += name;
    ret += "\":";
    ret += &serde_json::to_string(object).expect("Could not serialize object");
    ret += "}";
    ret
}
pub fn json_build_vec<T: Serialize>(tag: &str, name: &str, v: &Vec<T>) -> String {
    let mut ret = String::new();
    ret += &format!("{};", tag);
    ret += r#"{"#;
    ret += &format!("\"{}\":", name);
    ret += "[";
    
    /*
    let mapped =
        v.adv_map(|x| { ret += &serde_json::to_string(*x).expect("Could not serialize object"); ret += ","; },
                  |x| { ret += &serde_json::to_string(*x).expect("Could not serialize object"); });
    */
    let mut delms = {
        let mut tmp = vec![","; v.len() - 1];
        tmp.push("");
        tmp.into_iter()
    };

    for e in v {
        ret += &serde_json::to_string(&e).expect("Could not serialize object");
        ret += delms.next().unwrap();
    }
    ret += "]}";
    ret
}

/*
trait EMap<T> {
    fn adv_map<O>(&self, impl Fn(&T) -> O, impl Fn(&T) -> O) -> Vec<O>;
}

impl<T> EMap<T> for Vec<T> {
    fn adv_map<O>(&self, non_last: impl Fn(&T) -> O, last: impl Fn(&T) -> O) -> Vec<O> {
        let mut ret = vec![];
        for i in 0..self.len() - 1 {
            ret.push(non_last(&self[i]));
        }
        ret.push(last(self.last().unwrap()));
        ret
    }
}
*/
