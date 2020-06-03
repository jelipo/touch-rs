use std::collections::HashMap;

use async_std::net::TcpStream;

pub trait Connect {
    fn connect(&self);
}

pub struct Socks5<'a> {
    tcp_stream: &'a TcpStream
}

impl Connect for Socks5 {
    async fn connect(&self) {
        let x = self.tcp_stream;
    }
}

impl Socks5 {
    pub fn new(tcp: &TcpStream) -> Socks5 {
        Socks5 { tcp_stream: tcp }
    }
}