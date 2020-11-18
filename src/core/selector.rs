use crate::core::config::ConfigReader;
use crate::core::profile::{ConnectMode, ProtocalType};

pub struct ProtocalSelector {}

impl ProtocalSelector {
    pub fn select(config_reader: &ConfigReader) {
        let input = &config_reader.input;
        let input_mode = input.mode.as_ref().unwrap_or(&ConnectMode::Passive);
        match input.name {
            ProtocalType::Original => {}
            ProtocalType::Socks5 => {}
            ProtocalType::SsAes128Gcm |
            ProtocalType::SsAes256Gcm |
            ProtocalType::Chacha20Poly1305 => {}
        }
    }
}