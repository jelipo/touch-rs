pub mod ss_stream;
pub mod proxy;
mod connect;
pub mod socks5;
pub mod raw;


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AddressType {
    IPv4,
    Domain,
    IPv6,
}