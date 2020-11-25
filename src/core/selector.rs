use crate::core::config::ConfigReader;
use crate::core::profile::{ConnectMode, ProtocalType};

pub struct ProtocalSelector {}

impl ProtocalSelector {
    pub fn select(config_reader: &ConfigReader) {
        let input = &config_reader.input;
        let input_name = &config_reader.input.name;
        let input_mode = input.mode.as_ref().unwrap_or(&ConnectMode::Passive);
        match (input_name, input_mode) {
            (ProtocalType::Original, &ConnectMode::Passive) => {}
            (ProtocalType::Original, &ConnectMode::Active) => {}
            (ProtocalType::Socks5, &ConnectMode::Passive) => {}
            (ProtocalType::SsAes128Gcm, _) => {}
            (ProtocalType::SsAes256Gcm, _) => {}
            (ProtocalType::Chacha20Poly1305, _) => {}
            _ => { println!("Not support {:?}:{:?}", input_name, input_mode) }
        }
    }
}