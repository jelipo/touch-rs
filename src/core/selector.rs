use std::io;
use std::io::ErrorKind;

use crate::core::config::ConfigReader;
use crate::core::profile::{BaseActiveConfig, BasePassiveConfig, ConnectMode, ProtocalType, ProtocolConf, RawActiveConfig};
use crate::encrypt::aead::AeadType;
use crate::net::proxy::{InputProxy, OutputProxy};
use crate::net::raw::RawActive;
use crate::net::socks5::Socks5Passive;
use crate::net::ss_stream::{SsInputProxy, SsOutProxy};

pub struct ProtocolSelector {}

impl ProtocolSelector {
    pub async fn select(config_reader: &ConfigReader) -> io::Result<()> {
        let output_proxy = select_output(&config_reader.output)?;
        let mut input_proxy = select_input(&config_reader.input, output_proxy).await?;
        // Start proxy
        input_proxy.start().await?;
        Ok(())
    }
}

/// Select the output proxy and initialize it.
fn select_output(output: &ProtocolConf) -> io::Result<Box<dyn OutputProxy + Send>> {
    let output_name = &output.name;
    let output_mode = output.mode.as_ref().unwrap_or(&ConnectMode::Active);
    let output_proxy: Box<dyn OutputProxy + Send> = match *output_mode {
        ConnectMode::Active => {
            match output_name {
                // Shadowsocks AEAD
                ProtocalType::SsAes256Gcm | ProtocalType::SsAes128Gcm | ProtocalType::Chacha20Poly1305 => {
                    let config: BaseActiveConfig = serde_json::from_value(output.config.clone())?;
                    Box::new(SsOutProxy::new(
                        config.remote_host,
                        config.remote_port,
                        config.password.unwrap(),
                        &change_ss_type(output_name),
                    ))
                }
                ProtocalType::Raw => {
                    let config: RawActiveConfig = serde_json::from_value(output.config.clone())?;
                    Box::new(RawActive::new(config.dns)?)
                }
                //ProtocalType::Original => {}
                // ProtocalType::Socks5 => {}
                _ => return Err(unsupport_err(output_name, output_mode)),
            }
        }
        ConnectMode::Passive => match output_name {
            _ => return Err(unsupport_err(output_name, output_mode)),
        },
    };
    Ok(output_proxy)
}

///
async fn select_input(input_conf: &ProtocolConf, output_proxy: Box<dyn OutputProxy + Send>) -> io::Result<Box<dyn InputProxy>> {
    let input_name = &input_conf.name;
    let input_mode = input_conf.mode.as_ref().unwrap_or(&ConnectMode::Passive);
    let input_proxy: Box<dyn InputProxy + Send> = match *input_mode {
        ConnectMode::Passive => {
            let config: BasePassiveConfig = serde_json::from_value(input_conf.config.clone())?;
            match input_name {
                //ProtocolType::Original => {}
                ProtocalType::Socks5 => Box::new(Socks5Passive::new(&config, output_proxy).await?),
                ProtocalType::SsAes128Gcm | ProtocalType::SsAes256Gcm | ProtocalType::Chacha20Poly1305 => {
                    Box::new(SsInputProxy::new(change_ss_type(input_name), &config, output_proxy).await?)
                }
                _ => return Err(unsupport_err(input_name, input_mode)),
            }
        }
        ConnectMode::Active => match input_name {
            _ => return Err(unsupport_err(input_name, input_mode)),
        },
    };
    Ok(input_proxy)
}

fn unsupport_err(name: &ProtocalType, mode: &ConnectMode) -> io::Error {
    let err = format!("Not support type: {:?} - {:?}", name, mode);
    io::Error::new(ErrorKind::InvalidInput, err)
}

pub fn change_ss_type(t: &ProtocalType) -> AeadType {
    match t {
        ProtocalType::SsAes128Gcm => AeadType::AES128GCM,
        ProtocalType::SsAes256Gcm => AeadType::AES256GCM,
        ProtocalType::Chacha20Poly1305 => AeadType::Chacha20Poly1305,
        _ => AeadType::AES128GCM,
    }
}
