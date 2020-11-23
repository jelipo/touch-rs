use std::io::{Error, Result};

use async_std::io::ErrorKind;
use async_std::net::{TcpStream};
use async_std::prelude::*;

use crate::net::AddressType;
use crate::net::proxy::ProxyInfo;

/// Socks5 协议
pub struct Socks5Connector<'a> {
    tcp_stream: &'a mut TcpStream,
}

impl<'a> Socks5Connector<'a> {
    pub fn new(tcp: &'a mut TcpStream) -> Self {
        Self { tcp_stream: tcp }
    }

    fn method_size(socks5_head: &[u8]) -> Result<u8> {
        if socks5_head[0] != 5 {
            let err_str = format!("Unsupport Socks5 version:'{}", socks5_head[0]);
            return Err(Error::new(ErrorKind::ConnectionAborted, err_str));
        }
        Ok(socks5_head[1])
    }

    /// 检验协议头并建立连接的主要方法
    pub async fn check(&mut self) -> Result<ProxyInfo> {
        let mut head = vec![0u8; 2];
        self.tcp_stream.read_exact(&mut head).await?;
        let method_size = Socks5Connector::method_size(head.as_slice())?;
        //read client methods
        let mut first_method_arr = vec![0u8; method_size as usize];
        self.tcp_stream.read_exact(&mut first_method_arr).await?;
        //write server methods
        self.write_server_methods().await?;
        let info = self.read_address().await?;
        Ok(info)
    }

    /// 向client端写入server端支持的方法
    /// 当发送完成时返回true，反之false
    async fn write_server_methods(&mut self) -> Result<()> {
        let server_mthod = [5, 0];
        self.tcp_stream.write_all(&server_mthod).await
    }

    /// 从TCP流中读取发送过来的地址信息
    async fn read_address(&mut self) -> Result<ProxyInfo> {
        let mut address_head = [0u8; 4];
        self.tcp_stream.read(&mut address_head).await?;
        let address_type_byte = address_head[3];
        let address_type = AddressType::with_byte(address_type_byte)?;
        let address = match address_type {
            AddressType::IPv4 => self.read_ipv4_address().await,
            AddressType::Domain => self.read_domain_address().await,
            _ => return Err(Error::new(ErrorKind::InvalidInput, "不支持的地址类型")),
        };
        let port = self.read_port().await;
        self.write_connect_success(port);
        Ok(ProxyInfo {
            address_type,
            address: address?,
            port,
        })
    }

    /// 从TCP流中读取4个字节并返回
    async fn read_ipv4_address(&mut self) -> Result<Box<Vec<u8>>> {
        let mut ip_arr = Box::new(vec![0u8; 4]);
        self.tcp_stream.read_exact(ip_arr.as_mut()).await?;
        Ok(ip_arr)
    }

    /// 从TCP流中读取域名版的地址
    async fn read_domain_address(&mut self) -> Result<Box<Vec<u8>>> {
        let mut length_arr = [0u8; 1];
        self.tcp_stream.read_exact(&mut length_arr).await?;
        let length = length_arr[0];
        let mut domain_addr = Box::new(vec![0u8; length as usize]);
        self.tcp_stream.read_exact(domain_addr.as_mut()).await?;
        Ok(domain_addr)
    }

    /// 从TCP流中读取端口号
    async fn read_port(&mut self) -> u16 {
        let mut length_arr = [0u8; 2];
        self.tcp_stream.read_exact(&mut length_arr).await;
        u16::from_be_bytes(length_arr)
    }

    /// 向客户端写入连接成功的消息
    async fn write_connect_success(&mut self, port: u16) -> Result<()> {
        //连接成功的字节中，前几位是固定的
        const HEAD: [u8; 8] = [5, 0, 0, 1, 0, 0, 0, 0];
        self.tcp_stream.write_all(&HEAD).await;
        let port_arr = port.to_be_bytes();
        self.tcp_stream.write_all(&port_arr).await
    }
}