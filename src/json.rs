
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

#[test]
fn serialize_test() {
    use super::map::coord::RawCoord;
    let coord = RawCoord{x: 10., y: 20., z: 30.};
    let mut result = json_build("TestTag", "TestName", &coord);
    assert_eq!(result, r#"TestTag;{"TestName":{"x":10.0,"y":20.0,"z":30.0}}"#.to_string());

    let coords = vec![
        RawCoord{ x: 1., y: 2., z: 3.},
        RawCoord{ x: 4., y: 5., z: 6.},
        RawCoord{ x: 7., y: 8., z: 9.},
    ];
    result = json_build("TestVecTag", "TestVecName", &coords);
    assert_eq!(result, r#"TestVecTag;{"TestVecName":[{"x":1.0,"y":2.0,"z":3.0},{"x":4.0,"y":5.0,"z":6.0},{"x":7.0,"y":8.0,"z":9.0}]}"#);
}
