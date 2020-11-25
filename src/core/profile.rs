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
    /// Protocol name
    pub name: ProtocalType,
    /// Active or Passive mode
    pub mode: Option<ConnectMode>,
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

#[derive(Serialize, Deserialize, Debug)]
pub enum ProtocalType {
    #[serde(alias = "original")]
    Original,
    #[serde(alias = "socks5")]
    Socks5,
    #[serde(alias = "ss-aes-128-gcm")]
    SsAes128Gcm,
    #[serde(alias = "ss-aes-256-gcm")]
    SsAes256Gcm,
    #[serde(alias = "chacha20poly1305")]
    Chacha20Poly1305,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConnectMode {
    #[serde(alias = "active")]
    Active,
    #[serde(alias = "passive")]
    Passive,
}