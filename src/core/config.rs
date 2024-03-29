use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::result::Result::Err;

use log::error;

use crate::core::profile::{Profile, ProtocolConf};

pub struct ConfigReader {
    pub input: ProtocolConf,
    pub output: ProtocolConf,
}

/// Read the config file and deserialize it.
impl ConfigReader {
    pub fn read_config(path: &Path) -> io::Result<Self> {
        let profile = read_file(path)?;
        Ok(Self {
            input: profile.input,
            output: profile.output,
        })
    }
}

fn read_file(path: &Path) -> io::Result<Profile> {
    let file_max_size: u64 = 1 * 1024 * 1024;
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() > file_max_size {
        let err = format!("The file is too large , MAX_FILE_SZIE: {}KB", file_max_size / 1024);
        return Err(Error::new(ErrorKind::InvalidInput, err));
    }
    let result: serde_json::Result<Profile> = serde_json::from_reader(file);
    result.map_err(|e| {
        error!("Read file failed:{}", e);
        Error::new(ErrorKind::InvalidInput, "Read file failed.")
    })
}
