pub mod stream;
pub mod proxy;
mod connect;
mod socks5;


#[derive(Debug, PartialEq)]
pub enum AddressType {
    IPv4,
    Domain,
    IPv6,
}