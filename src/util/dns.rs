use std::io::{Error, ErrorKind};
use std::io;
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct Dns {}

impl Dns {
    pub fn change_ipv4(ipv4_str: &str) -> io::Result<Ipv4Addr> {
        Ipv4Addr::from_str(ipv4_str.as_str()).or_else(|e| {
            let eror_str = format!("'{}' is not a DNS address", ipv4_str);
            Err(Error::new(ErrorKind::InvalidInput, eror_str))
        })
    }
}