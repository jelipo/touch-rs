use std::error::Error;
use std::str::FromStr;

use async_std::io;
use async_std::io::ErrorKind;
use async_std::net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use async_std::prelude::*;

use crate::socks::consts::{AddressHeader, AddressType, Command, SocksVersion};

const SOCKS5_SUPPORT: [u8; 2] = [5, 0];

pub struct Socks5<'a> {
    tcp_stream: &'a mut TcpStream
}

impl<'a> Socks5<'a> {
    pub fn new(tcp: &'a mut TcpStream) -> Self {
        Self { tcp_stream: tcp }
    }

    fn check_head(socks5_head: Vec<u8>) -> (bool, u8) {
        if socks5_head[0] != 5 {
            println!("不支持的socks5协议版本");
            return (false, 0u8);
        }

        return (true, socks5_head[1]);
    }

    pub async fn connect(&mut self) -> Option<TcpStream> {
        return self.start_connect().await;
    }

    /// 检验协议头并建立连接的主要方法
    async fn start_connect(&mut self) -> Option<TcpStream> {
        let mut head = vec![0u8; 2];
        let read = self.tcp_stream.read(&mut head);
        if read.await.unwrap() == 0 { return None; }
        let check_result = Socks5::check_head(head);
        if !check_result.0 { return None; }
        //read client methods
        let method_size = check_result.1;
        let mut first_method_arr = vec![0u8; method_size as usize];
        let end = self.tcp_stream.read(&mut first_method_arr).await;
        let i = end.unwrap();
        //write server methods
        if !self.write_server_methods().await {
            println!("方法发送失败");
            return None;
        }
        let address_header = self.read_address().await;
        let remote_stream = match address_header.cmd {
            Command::Connect => self.connect_tcp_remote(
                &address_header.address, &address_header.port).await,
            Command::Bind => Result::Err(
                io::Error::new(ErrorKind::InvalidInput, "暂不支持BIND方法")),
            Command::UdpAssociate => Err(ErrorKind::InvalidInput)
        };
        let local_addr = remote_stream.local_addr().unwrap();
        self.write_connect_success(local_addr).await;
        return Option::Some(remote_stream.unwrap());
    }

    /// 向client端写入server端支持的方法
    /// 当发送完成时返回true，反之false
    async fn write_server_methods(&mut self) -> bool {
        let server_mthod = SOCKS5_SUPPORT;
        let write = self.tcp_stream.write(&server_mthod);
        return write.await.unwrap() != 0;
    }

    /// 从TCP流中读取发送过来的地址信息
    async fn read_address(&mut self) -> AddressHeader {
        let mut address_head = [0u8; 4];
        let end = self.tcp_stream.read(&mut address_head).await;
        let address_type_byte = address_head[3];
        let address_type = AddressType::with_byte(address_type_byte)
            .expect("未知的类型");
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
            port: self.read_port().await,
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

    /// 从TCP流中读取端口号
    async fn read_port(&mut self) -> u16 {
        let mut length_arr = [0u8; 2];
        let read = self.tcp_stream.read(&mut length_arr).await;
        return u16::from_be_bytes(length_arr);
    }

    /// 向客户端写入连接成功的消息
    async fn write_connect_success(&mut self, local_addr: SocketAddr) {
        let mut head: [u8; 9] = [5, 0, 0, 1, 4, 0, 0, 0, 0];
        self.tcp_stream.write(&mut head).await;
        let mut port_vec = local_addr.port().to_be_bytes().to_vec();
        self.tcp_stream.write(&mut port_vec).await;
    }

    async fn connect_tcp_remote(&mut self, host: &String, port: &u16) -> io::Result<TcpStream> {
        let address = host.to_string() + ":" + port.to_string().as_ref();
        let address_str = address.as_str();
        println!("host {}", address_str);
        return TcpStream::connect(address_str).await;
    }

    async fn connect_udp_remote(&mut self, host: &String, port: &u16) -> io::Result<TcpStream> {
        let address = host.to_string() + ":" + port.to_string().as_ref();
        let address_str = address.as_str();
        println!("host {}", address_str);
        UdpSocket::connect(address_str).await.let
    }
}