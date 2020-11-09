pub mod stream;
mod proxy;
mod connect;


#[derive(Debug, PartialEq)]
pub enum AddressType {
    IPv4,
    Domain,
    IPv6,
}