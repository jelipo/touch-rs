use std::collections::hash_map::RandomState;
use std::collections::HashMap;

pub struct Socks5Protocal {
    pub a: u8
}

impl Socks5Protocal {
    //!
    pub fn socks5_test() -> String {
        return "hello".to_string();
    }

    pub fn new() -> Socks5Protocal {
        return Socks5Protocal {
            a: 9
        };
    }
}
