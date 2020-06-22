use std::io;
use std::io::Error;

use async_std::io::ErrorKind;

pub struct AddressHeader {
    pub socks_version: SocksVersion,
    pub cmd: Command,
    pub address_type: AddressType,
    pub address: String,
    pub port: u16,
}

pub enum Command {
    //0x01 连接
    Connect,
    //0x02 端口监听
    Bind,
    //0x03 使用UDP
    UdpAssociate,
}

impl Command {
    pub fn with_byte(cmd: u8) -> Result<Command, Error> {
        match cmd {
            0x01 => Ok(Command::Connect),
            0x02 => Ok(Command::Bind),
            0x03 => Ok(Command::UdpAssociate),
            _ => Err(Error::new(ErrorKind::InvalidInput, "暂不支持此方法")),
        }
    }
}

/// SOCKS的版本协议，本程序只实现V5版本
pub enum SocksVersion {
    V5,
    V4,
}

impl SocksVersion {
    pub fn with_byte(version: u8) -> SocksVersion {
        match version {
            0x05 => SocksVersion::V5,
            0x04 => SocksVersion::V4,
            _ => SocksVersion::V5,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AddressType {
    //0x01
    IPv4,
    //0x03
    Domain,
    //0x04
    IPv6,
}

impl AddressType {
    pub fn with_byte(address_type: u8) -> io::Result<AddressType> {
        match address_type {
            0x01 => Ok(AddressType::IPv4),
            0x03 => Ok(AddressType::Domain),
            0x04 => Ok(AddressType::IPv6),
            _ => Err(Error::new(ErrorKind::InvalidInput, "暂不支持BIND方法")),
        }
    }
}
