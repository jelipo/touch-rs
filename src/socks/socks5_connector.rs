use std::io::{Error, Result};

use async_std::io::Read;
use async_std::io::ErrorKind;
use async_std::net::{Ipv4Addr, SocketAddr, TcpStream};
use async_std::prelude::*;

use crate::net::AddressType;
use crate::socks::consts::{AddressHeader, Command, SocksVersion};

/// Socks5 协议连接器
pub struct Socks5Connector<'a> {
    tcp_stream: &'a mut TcpStream,
}

impl<'a> Socks5Connector<'a> {
    pub fn new(tcp: &'a mut TcpStream) -> Self {
        Self { tcp_stream: tcp }
    }

    fn check_head(socks5_head: Vec<u8>) -> Result<u8> {
        let socks_version = socks5_head[0];
        if socks_version != 5 {
            return Err(Error::new(
                ErrorKind::ConnectionAborted,
                format!("Unsupport socks version:'{}", socks_version),
            ));
        }
        return Ok(socks5_head[1]);
    }

    pub async fn connect(&mut self) -> Result<TcpStream> {
        return self.start_connect().await;
    }

    /// 检验协议头并建立连接的主要方法
    async fn start_connect(&mut self) -> Result<TcpStream> {
        let mut head = vec![0u8; 2];
        let read = self.tcp_stream.read(&mut head).await?;
        if read == 0 {
            return Err(Error::new(ErrorKind::ConnectionAborted, "Connection closed."));
        }
        let method_size = Socks5Connector::check_head(head)?;
        //read client methods
        let mut first_method_arr = vec![0u8; method_size as usize];
        if 0 == self.tcp_stream.read(&mut first_method_arr).await? {
            return Err(Error::new(ErrorKind::ConnectionAborted, "Connection closed."));
        }
        //write server methods
        if !self.write_server_methods().await? {
            return Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Method failed to send.",
            ));
        }
        let address_header = self.read_address().await?;
        let remote_stream = match address_header.cmd {
            Command::Connect => {
                self.connect_tcp_remote(&address_header.address, &address_header.port).await?
            }
            Command::Bind => {
                return Err(Error::new(ErrorKind::InvalidInput, "Not support 'BIND'."));
            }
            Command::UdpAssociate => {
                return Err(Error::new(ErrorKind::InvalidInput, "Not support 'UdpAssociate'."));
            }
        };
        let local_addr = remote_stream.local_addr()?;
        self.write_connect_success(local_addr).await;
        return Result::Ok(remote_stream);
    }

    /// 向client端写入server端支持的方法
    /// 当发送完成时返回true，反之false
    async fn write_server_methods(&mut self) -> Result<bool> {
        let server_mthod = [5, 0];
        let write = self.tcp_stream.write(&server_mthod).await?;
        return Ok(write != 0);
    }

    /// 从TCP流中读取发送过来的地址信息
    async fn read_address(&mut self) -> Result<AddressHeader> {
        let mut address_head = [0u8; 4];
        self.tcp_stream.read(&mut address_head).await?;
        let address_type_byte = address_head[3];
        let address_type = AddressType::with_byte(address_type_byte)?;
        let address = match address_type {
            AddressType::IPv4 => self.read_ipv4_address().await,
            AddressType::Domain => self.read_domain_address().await,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "不支持的地址类型")),
        };
        return Ok(AddressHeader {
            socks_version: SocksVersion::with_byte(address_head[0]),
            cmd: Command::with_byte(address_head[1])?,
            address_type,
            address: address?,
            port: self.read_port().await,
        });
    }

    /// 从TCP流中读取4个字节并返回为字符串
    async fn read_ipv4_address(&mut self) -> Result<String> {
        let ip_arr = [0u8; 4];
        return Ok(Ipv4Addr::from(ip_arr).to_string());
    }

    /// 从TCP流中读取域名版的地址
    async fn read_domain_address(&mut self) -> Result<String> {
        let mut length_arr = [0u8; 1];
        let read = self.tcp_stream.read(&mut length_arr).await?;
        if read == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Connection closed."));
        }
        let length = length_arr[0];
        let mut domain_addr = vec![0u8; length as usize];
        let end = self.tcp_stream.read(&mut domain_addr).await?;
        if end == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Connection closed."));
        }
        return match String::from_utf8(domain_addr) {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::new(ErrorKind::ConnectionAborted, "格式转换失败")),
        };
    }

    /// 从TCP流中读取端口号
    async fn read_port(&mut self) -> u16 {
        let mut length_arr = [0u8; 2];
        let read = self.tcp_stream.read(&mut length_arr).await;
        return u16::from_be_bytes(length_arr);
    }

    /// 向客户端写入连接成功的消息
    async fn write_connect_success(&mut self, local_addr: SocketAddr) {
        //连接成功的字节中，前几位是固定的
        const HEAD: [u8; 8] = [5, 0, 0, 1, 0, 0, 0, 0];
        self.tcp_stream.write(&HEAD).await;
        let port_arr = local_addr.port().to_be_bytes();
        self.tcp_stream.write(&port_arr).await;
    }

    /// 使用TCP连接远程地址
    async fn connect_tcp_remote(&mut self, host: &String, port: &u16) -> Result<TcpStream> {
        let address = format!("{}:{}", host, port);
        return TcpStream::connect(address.as_str()).await;
    }
}
