
use std::marker::Send;

fn main() {

}

struct MyStruct;

unsafe impl Send for MyStruct {
}
