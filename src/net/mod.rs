pub mod stream;
mod proxy;
mod connect;
mod socks5_stream;


#[derive(Debug, PartialEq)]
pub enum AddressType {
    IPv4,
    Domain,
    IPv6,
}