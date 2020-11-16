use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind};
use std::io::Read;
use std::path::Path;
use std::result::Result::Err;
use async_std::net::TcpStream;
use log::{info, trace, warn};


use crate::core::profile::{Profile, ProtocolConf};

struct ConfigReader {
    pub input: ProtocolConf,
    pub output: ProtocolConf,
}

const FILE_MAX_SIZE: u64 = 1 * 1024 * 1024;

/// Read the config file and deserialize it.
impl ConfigReader {
    pub fn read_config(path: &Path) -> io::Result<Self> {
        let profile = read_file(path)?;
        std::
        Ok(Self {
            input: profile.input,
            output: profile.output,
        })
    }
}

fn read_file(path: &Path) -> io::Result<Profile> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() > FILE_MAX_SIZE {
        let err = format!("The file is too large,MAX_FILE_SZIE:{}KB", FILE_MAX_SIZE / 1024);
        return Err(Error::new(ErrorKind::InvalidInput, err));
    }
    let result: serde_json::Result<Profile> = serde_json::from_reader(file);
    result.or(Err(Error::new(ErrorKind::InvalidInput, "It's not a JSON file.")))
}