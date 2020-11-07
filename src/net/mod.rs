mod stream;
mod proxy;


#[derive(Debug, PartialEq)]
pub enum AddressType {
    IPv4,
    Domain,
    IPv6,
}