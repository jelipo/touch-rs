use std::collections::HashMap;

use async_std::net::TcpStream;

pub struct Socks5<'a> {
    tcp_stream: &'a TcpStream
}

impl Socks5 {
    pub async fn connect(&self) {
        let mut head = [0u8; 2];
        let mut stream = self.tcp_stream;
        let read = stream.read(&head);
        if read.await.unwrap() == 0 {
            stream.close();
            return;
        }
        let check_result = Socks5::check_head(head);
        if !check_result.0 {
            stream.close();
            return;
        }
        let method_size = check_result.1;
        let mut first_method_arr = [0u8; method_size];
        let first_method_arr_read = stream.read(&first_method_arr);
    }
}

impl Socks5 {
    pub fn new(tcp: &TcpStream) -> Socks5 {
        Socks5 { tcp_stream: tcp }
    }

    fn check_head(socks5_head: [u8; 2]) -> (bool, u8) {
        if socks5_head[0] != 5 {
            println!("不支持的socks5协议版本");
            return (false, 0u8);
        }
        return (false, socks5_head[1]);
    }
}