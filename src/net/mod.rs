mod dns;
pub mod http;
pub mod proxy;
pub mod raw;
pub mod socks5;
pub mod ss_stream;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AddressType {
    IPv4,
    Domain,
    IPv6,
}
