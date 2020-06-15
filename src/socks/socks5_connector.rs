use std::io::Error;

use async_std::io;
use async_std::io::ErrorKind;
use async_std::net::{Ipv4Addr, SocketAddr, TcpStream};
use async_std::prelude::*;

use crate::socks::consts::{AddressHeader, AddressType, Command, SocksVersion};

/// Socks5 协议连接器
pub struct Socks5Connector<'a> {
    tcp_stream: &'a mut TcpStream
}

impl<'a> Socks5Connector<'a> {
    pub fn new(tcp: &'a mut TcpStream) -> Self {
        Self { tcp_stream: tcp }
    }

    fn check_head(socks5_head: Vec<u8>) -> io::Result<u8> {
        if socks5_head[0] != 5 {
            return Err(Error::new(ErrorKind::ConnectionAborted, "不支持的socks5协议版本"));
        }
        return Ok(socks5_head[1]);
    }

    pub async fn connect(&mut self) -> io::Result<TcpStream> {
        return self.start_connect().await;
    }

    /// 检验协议头并建立连接的主要方法
    async fn start_connect(&mut self) -> io::Result<TcpStream> {
        let mut head = vec![0u8; 2];
        let read = self.tcp_stream.read(&mut head).await?;
        if read == 0 {
            return Err(Error::new(ErrorKind::ConnectionAborted, "连接可能已经断开"));
        }
        let method_size = Socks5Connector::check_head(head)?;
        //read client methods
        let mut first_method_arr = vec![0u8; method_size as usize];
        if 0 == self.tcp_stream.read(&mut first_method_arr).await? {
            return Err(Error::new(ErrorKind::ConnectionAborted, "读取失败"));
        }
        //write server methods
        if !self.write_server_methods().await? {
            return Err(Error::new(ErrorKind::ConnectionAborted, "方法发送失败"));
        }
        let address_header = self.read_address().await?;
        let remote_stream_result = match address_header.cmd {
            Command::Connect => self.connect_tcp_remote(
                &address_header.address, &address_header.port).await,
            Command::Bind => Result::Err(
                Error::new(ErrorKind::InvalidInput, "暂不支持BIND方法")),
            Command::UdpAssociate => Result::Err(
                Error::new(ErrorKind::InvalidInput, "暂不支持UdpAssociate方法")),
        };
        let remote_stream = remote_stream_result?;
        let local_addr = remote_stream.local_addr()?;
        self.write_connect_success(local_addr).await;
        return Result::Ok(remote_stream);
    }

    /// 向client端写入server端支持的方法
    /// 当发送完成时返回true，反之false
    async fn write_server_methods(&mut self) -> io::Result<bool> {
        let server_mthod = [5, 0];
        let write = self.tcp_stream.write(&server_mthod).await?;
        return Ok(write != 0);
    }

    /// 从TCP流中读取发送过来的地址信息
    async fn read_address(&mut self) -> io::Result<AddressHeader> {
        let mut address_head = [0u8; 4];
        self.tcp_stream.read(&mut address_head).await?;
        let address_type_byte = address_head[3];
        let address_type_result = AddressType::with_byte(address_type_byte);
        if let Err(err) = address_type_result {
            return Err(err);
        };
        let address_type = address_type_result?;
        let address = match address_type {
            AddressType::IPv4 => self.read_ipv4_address().await,
            AddressType::Domain => self.read_domain_address().await,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "不支持的地址类型")),
        };
        return Ok(AddressHeader {
            socks_version: SocksVersion::with_byte(address_head[0]),
            cmd: Command::with_byte(address_head[1]),
            address_type,
            address: address?,
            port: self.read_port().await,
        });
    }

    /// 从TCP流中读取4个字节并返回为字符串
    async fn read_ipv4_address(&mut self) -> io::Result<String> {
        let mut ip_arr = [0u8; 4];
        self.tcp_stream.read(&mut ip_arr).await;
        return Ok(Ipv4Addr::new(ip_arr[0], ip_arr[1], ip_arr[2],
                                ip_arr[3]).to_string());
    }

    /// 从TCP流中读取域名版的地址
    async fn read_domain_address(&mut self) -> io::Result<String> {
        let mut length_arr = [0u8; 1];
        let read = self.tcp_stream.read(&mut length_arr).await?;
        if read == 0 { return Err(Error::new(ErrorKind::InvalidInput, "连接可能断开")); }
        let length = length_arr[0];
        let mut domain_addr = vec![0u8; length as usize];
        let end = self.tcp_stream.read(&mut domain_addr).await?;
        if end == 0 { return Err(Error::new(ErrorKind::InvalidInput, "连接可能断开")); }
        return match String::from_utf8(domain_addr) {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::new(
                ErrorKind::ConnectionAborted, "格式转换失败"))
        };
    }

    /// 从TCP流中读取端口号
    async fn read_port(&mut self) -> u16 {
        let mut length_arr = [0u8; 2];
        self.tcp_stream.read(&mut length_arr).await;
        return u16::from_be_bytes(length_arr);
    }

    /// 向客户端写入连接成功的消息
    async fn write_connect_success(&mut self, local_addr: SocketAddr) {
        //连接成功的字节中，前几位是固定的
        let mut head = [5, 0, 0, 1, 0, 0, 0, 0];
        self.tcp_stream.write(&mut head).await;
        let port_arr = local_addr.port().to_be_bytes();
        self.tcp_stream.write(&port_arr).await;
    }

    async fn connect_tcp_remote(&mut self, host: &String, port: &u16) -> io::Result<TcpStream> {
        let address = host.to_string() + ":" + port.to_string().as_ref();
        let address_str = address.as_str();
        println!("host {}", address_str);
        return TcpStream::connect(address_str).await;
    }
}