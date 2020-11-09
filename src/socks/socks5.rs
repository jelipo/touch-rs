use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::vec::IntoIter;

pub struct Socks5 {}

impl Socks5 {
    pub fn read_to_socket_addrs(bytes: &[u8]) -> String {
        let addr_type = bytes[0];
        let socket_addr = match addr_type {
            0x01 => {
                let ipv4_addr = Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]);
                let v4 = SocketAddrV4::new(ipv4_addr, u16::from_be_bytes([bytes[5], bytes[6]]));
                v4.to_string()
            }
            0x04 => {
                let mut ipv6_arr = [0u8; 16];
                ipv6_arr.copy_from_slice(&bytes[1..17]);
                let port = u16::from_be_bytes([bytes[17], bytes[18]]);
                let v6 = SocketAddrV6::new(Ipv6Addr::from(ipv6_arr), port, 0, 0);
                v6.to_string()
            }
            _ => {
                let domain_len = bytes[1];
                let mut domain_buf = vec![0u8; domain_len as usize];
                let i: usize = (domain_len + 2) as usize;
                domain_buf.copy_from_slice(&bytes[2..i]);
                let domain = String::from_utf8(domain_buf)
                    .or(Err(Error::new(ErrorKind::InvalidInput, ""))).unwrap();
                let port = u16::from_be_bytes([bytes[(domain_len + 2) as usize], bytes[(domain_len + 3) as usize]]);
                format!("{}:{}", domain, port)
            }
        };
        socket_addr
    }
}


#[cfg(test)]
mod tests {
    use crate::socks::socks5::Socks5;

    #[test]
    fn test() {
        let mut bytes = [0u8; 128];
        bytes[0] = 4;
        let string = Socks5::read_to_socket_addrs(&bytes);
        println!("{}", string);
    }
}