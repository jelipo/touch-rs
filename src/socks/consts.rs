pub struct AddressHeader {
    pub socks_version: SocksVersion,
    pub cmd: Command,
    pub address_type:,
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
    pub fn with_byte(cmd: u8) -> Command {
        match cmd {
            1 => Command::Connect,
            2 => Command::Bind,
            3 => Command::UdpAssociate,
            _ => {}
        }
    }
}

/// SOCKS的版本协议，本程序只实现SOCKS5
pub enum SocksVersion {
    V5,
    V4,
}

impl SocksVersion {
    pub fn with_byte(version: u8) -> SocksVersion {
        match version {
            5 => SocksVersion::V5,
            4 => SocksVersion::V4,
            _ => {}
        }
    }
}


pub enum AddressType {
    //0x01
    IPv4,
    //0x03
    Domain,
    //0x04
    IPv6,
}

impl AddressType {
    pub fn with_byte(address_type: u8) -> AddressType {
        match address_type {
            1 => AddressType::IPv4,
            3 => AddressType::Domain,
            4 => AddressType::IPv6,
            _ => {}
        }
    }
}

