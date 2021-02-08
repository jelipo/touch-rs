use std::io::{Error, ErrorKind, Result};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::net::AddressType;
use crate::net::proxy::ProxyInfo;
use std::borrow::{BorrowMut, Borrow};
use std::slice::SliceIndex;

/// Socks5 协议
pub struct Socks5Server<'a> {
    tcp_stream: &'a mut TcpStream,
}

impl<'a> Socks5Server<'a> {
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
    pub async fn accept_check(&mut self) -> Result<ProxyInfo> {
        let mut head = vec![0u8; 2];
        self.tcp_stream.read_exact(&mut head).await?;
        let method_size = Socks5Server::method_size(head.as_slice())?;
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
        let port = self.read_port().await?;
        self.write_success_connect(port).await?;
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
        let length = self.tcp_stream.read_u8().await?;
        let mut domain_addr = Box::new(vec![0u8; length as usize]);
        self.tcp_stream.read_exact(domain_addr.as_mut()).await?;
        Ok(domain_addr)
    }

    /// 从TCP流中读取端口号
    async fn read_port(&mut self) -> Result<u16> {
        let len = self.tcp_stream.read_u16().await?;
        Ok(len)
    }

    /// 向客户端写入连接成功的消息
    async fn write_success_connect(&mut self, port: u16) -> Result<()> {
        //连接成功的字节中，前8位是固定的，后两位是端口
        let mut head: [u8; 10] = [5, 0, 0, 1, 0, 0, 0, 0, 0, 0];
        let port_arr = port.to_be_bytes();
        head[8..10].copy_from_slice(&port_arr);
        self.tcp_stream.write_all(&head).await
    }
}

pub struct Sock5ClientConnector<'a> {
    tcp_stream: &'a mut TcpStream,
}

impl<'a> Sock5ClientConnector<'a> {
    pub fn new(tcp: &'a mut TcpStream) -> Self {
        Self { tcp_stream: tcp }
    }

    pub async fn try_connect(&mut self, proxy_info: &ProxyInfo) -> Result<()> {
        let first = [5u8, 1, 0];
        self.tcp_stream.write_all(&first).await?;
        //  read server support info
        let mut first_read = [0u8; 2];
        let _read = self.tcp_stream.read_exact(&mut first_read).await?;
        if first_read[0] != 5 || first_read[1] != 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Connect socks5 server error."));
        }

        let (socks5_type_byte, socks5_addr_bytes) = match proxy_info.address_type {
            AddressType::IPv4 => {
                let mut bytes = vec![0u8; 4];
                bytes.copy_from_slice(&proxy_info.address[..4]);
                (1u8, bytes)
            }
            AddressType::Domain => {
                let mut domain_bytes = vec![0u8; 1 + proxy_info.address.len()];
                domain_bytes[0] = proxy_info.address.len() as u8;
                domain_bytes[1..].copy_from_slice(&proxy_info.address);
                (3u8, domain_bytes)
            }
            AddressType::IPv6 => {
                let mut bytes = vec![0u8; 16];
                bytes.copy_from_slice(&proxy_info.address[..16]);
                (4u8, bytes)
            }
        };
        // Write proxy info
        let fixed = [5, 1, 0, socks5_type_byte];
        let mut second_write = vec![0u8; 4 + socks5_addr_bytes.len() + 2];
        second_write[..4].copy_from_slice(&fixed);
        second_write[4..4 + socks5_addr_bytes.len()].copy_from_slice(&socks5_addr_bytes);
        second_write[4 + socks5_addr_bytes.len()..].copy_from_slice(&proxy_info.port.to_be_bytes());
        self.tcp_stream.write_all(&second_write).await?;
        //  read connect success info
        let mut address_head = [0u8; 4];
        self.tcp_stream.read(&mut address_head).await?;
        if address_head[0] != 5 || address_head[1] != 0 || address_head[2] != 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Connect socks5 server failed."));
        }
        let address_type_byte = address_head[3];
        let address_type = AddressType::with_byte(address_type_byte)?;
        let address_len = match address_type {
            AddressType::IPv4 => 4 as usize,
            AddressType::Domain => self.tcp_stream.read_u8().await? as usize,
            AddressType::IPv6 => 16 as usize,
        };
        let mut addr_port_vec = vec![0u8; address_len + 2];
        let _size = self.tcp_stream.read_exact(&mut addr_port_vec).await?;
        Ok(())
    }
}
