use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::vec::IntoIter;

pub struct Socks5 {}

impl Socks5 {
    /// # Return value
    /// - `String` Host address
    /// - `u8` Number of bytes read
    pub fn read_to_socket_addrs(bytes: &[u8]) -> (String, usize) {
        let addr_type = bytes[0];
        match addr_type {
            0x01 => {
                let port = u16::from_be_bytes([bytes[5], bytes[6]]);
                (format!("{}.{}.{}.{}:{}", bytes[1], bytes[2], bytes[3], bytes[4], port), 7)
            }
            0x04 => {
                let mut ipv6_arr = [0u8; 16];
                ipv6_arr.copy_from_slice(&bytes[1..17]);
                let port = u16::from_be_bytes([bytes[17], bytes[18]]);
                let v6 = SocketAddrV6::new(Ipv6Addr::from(ipv6_arr), port, 0, 0);
                (v6.to_string(), 19)
            }
            _ => {
                let domain_len = bytes[1] as usize;
                let cow = String::from_utf8_lossy(&bytes[2..(domain_len + 2)]);
                let port = u16::from_be_bytes([bytes[domain_len + 2], bytes[domain_len + 3]]);
                (format!("{}:{}", cow, port), 4 + domain_len)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::socks::socks5::Socks5;

    #[test]
    fn test() {
        let mut bytes = [0u8; 128];
        bytes[0] = 1;
        let string = Socks5::read_to_socket_addrs(&bytes);
        println!("{}", string.0);
    }
}