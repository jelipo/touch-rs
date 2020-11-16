use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    /// Custom DNS server
    pub dns: Option<String>,
    pub input: ProtocolConf,
    pub output: ProtocolConf,
}

#[derive(Serialize, Deserialize)]
pub struct ProtocolConf {
    /// Protocal name
    pub name: String,
    /// Active or Passive mode
    pub mode: Option<String>,
    /// Config
    pub config: Value,
}

/// The base config about active connection
#[derive(Serialize, Deserialize)]
pub struct BaseActiveConfig {
    /// Remote address , IPv4/IPv6/Domain
    pub remote_host: String,

    pub remote_port: u16,
    /// It's an `optional field`, but is `required` for some protocols
    pub password: Option<String>,
}

/// The base config about passive connection
#[derive(Serialize, Deserialize)]
pub struct BasePassiveConfig {
    /// Local address , IPv4/IPv6
    pub local_host: String,

    pub local_port: u16,
    /// It's an `optional field`, but is `required` for some protocols
    pub password: Option<String>,
}

pub enum InputType {
    Original,
    Socks5,
    SsAes128Gcm,
    SsAes256Gcm,
}