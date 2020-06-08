use async_std::net::{Ipv4Addr, TcpStream};
use async_std::prelude::*;

use crate::socks::consts::{AddressHeader, AddressType, Command, SocksVersion};

const SOCKS5_SUPPORT: [u8; 2] = [5, 0];

pub struct Socks5 {
    tcp_stream: TcpStream
}

impl Socks5 {
    pub fn new(tcp: TcpStream) -> Socks5 {
        Socks5 { tcp_stream: tcp }
    }

    fn check_head(socks5_head: Vec<u8>) -> (bool, u8) {
        if socks5_head[0] != 5 {
            println!("不支持的socks5协议版本");
            return (false, 0u8);
        }
        return (true, socks5_head[1]);
    }

    pub async fn connect(&mut self) {
        self.start_connect();
    }

    async fn start_connect(&mut self) -> bool {
        let mut head = vec![0u8; 2];
        let read = self.tcp_stream.read(&mut head);
        if read.await.unwrap() == 0 { return false; }
        let check_result = Socks5::check_head(head);
        if !check_result.0 { return false; }
        //read client methods
        let method_size = check_result.1;
        let mut first_method_arr = vec![0u8; method_size as usize];
        let end = self.tcp_stream.read(&mut first_method_arr).await;
        let i = end.unwrap();
        println!("收到方法：{:?}", first_method_arr);
        //write server methods
        if !self.write_server_methods().await {
            println!("方法发送失败");
            return false;
        }
        let address_header = self.read_address().await;
        return true;
    }

    /// 向client端写入server端支持的方法
    /// 当发送完成时返回true，反之false
    async fn write_server_methods(&mut self) -> bool {
        let mut writer = &self.tcp_stream;
        let server_mthod = SOCKS5_SUPPORT;
        let write = writer.write(&server_mthod);
        return write.await.unwrap() != 0;
    }

    /// 从TCP流中读取发送过来的地址信息
    async fn read_address(&mut self) -> AddressHeader {
        let mut address_head = [0u8; 4];
        let end = self.tcp_stream.read(&mut address_head).await;
        let address_type = AddressType::with_byte(address_head[3]);
        let address = match address_type {
            AddressType::IPv4 => self.read_ipv4_address().await,
            AddressType::Domain => self.read_domain_address().await,
            _ => None,
        };
        return AddressHeader {
            socks_version: SocksVersion::with_byte(address_head[0]),
            cmd: Command::with_byte(address_head[1]),
            address_type,
            address: address.unwrap(),
        };
    }

    /// 从TCP流中读取4个字节并返回为字符串
    async fn read_ipv4_address(&mut self) -> Option<String> {
        let mut ip_arr = [0u8; 4];
        let read = self.tcp_stream.read(&mut ip_arr).await;
        return Option::Some(Ipv4Addr::new(ip_arr[0], ip_arr[1], ip_arr[2],
                                          ip_arr[3]).to_string());
    }

    /// 从TCP流中读取域名版的地址
    async fn read_domain_address(&mut self) -> Option<String> {
        let mut length_arr = [0u8; 1];
        let read = self.tcp_stream.read(&mut length_arr).await;
        let length = length_arr[0];
        let mut domain_addr = vec![0u8; length as usize];
        let end = self.tcp_stream.read(&mut domain_addr).await;
        return String::from_utf8(domain_addr).ok();
    }
}