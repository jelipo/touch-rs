use serde_json::Value;
use serde::{Deserialize, Serialize};

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


pub enum InputType {
    Original,
    Socks5,
    SsAes128Gcm,
    SsAes256Gcm,
}