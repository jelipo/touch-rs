use std::collections::HashMap;
use std::env::var;
use std::time::SystemTime;

use crate::net::protocol::Socks5Protocal;
use std::borrow::Borrow;

mod net;

fn main() {
    let i = 1;
    let map = HashMap::new();
    let protocal = Socks5Protocal::new();

    let string = Socks5Protocal::socks5_test();
    println!("Hello world!")
}
