use std::io;

use async_std::io::ErrorKind;
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_trait::async_trait;

use crate::core::profile::ProtocalType;
use crate::encrypt::aead::AeadType;
use crate::encrypt::error::EncryptError;
use crate::encrypt::ss::ss_aead::SsAead;
use crate::net::proxy::{ProxyReader, ProxyWriter};

pub struct SsStreamReader {
    stream: TcpStream,
    password: Vec<u8>,
    aead_type: ProtocalType,
    ss_aead: Option<SsAead>,
    ss_len_buf: [u8; 18],
}

impl SsStreamReader {
    pub fn new(stream: TcpStream, password: &[u8], aead_type: ProtocalType) -> Self {
        SsStreamReader {
            stream,
            password: password.to_vec(),
            aead_type,
            ss_aead: None,
            ss_len_buf: [0u8; 18],
        }
    }
}

#[async_trait]
impl ProxyReader for SsStreamReader {
    async fn read(&mut self) -> io::Result<Vec<u8>> {
        // Check if this is the first read. If first read,creat the SsAead.
        if self.ss_aead.is_none() {
            let mut salt = [0u8; 32];
            self.stream.read_exact(&mut salt).await?;
            let ss_aead = SsAead::new(&salt, self.password.as_slice(), &self.aead_type)
                .or_else(|e| { Err(change_error(e)) })?;
            self.ss_aead = Some(ss_aead)
        }
        // Read bytes and decrypt byte
        self.stream.read_exact(&mut self.ss_len_buf).await?;
        let mut aead = self.ss_aead.as_mut().expect("");
        // Read
        let len_vec = decrypt(&self.ss_len_buf, aead)?;
        let len = u16::from_be_bytes([len_vec[0], len_vec[1]]);
        let mut en_data = vec![0u8; (len + 16) as usize];
        self.stream.read_exact(&mut en_data).await?;
        decrypt(en_data.as_slice(), aead)
    }
}


pub struct SsStreamWriter {
    stream: TcpStream,
    password: Vec<u8>,
    aead_type: ProtocalType,
    ss_aead: Option<SsAead>,
    ss_len_buf: [u8; 18],
}

impl SsStreamWriter {
    pub fn new(stream: TcpStream, password: &[u8], aead_type: ProtocalType) -> Self {
        SsStreamWriter {
            stream,
            password: password.to_vec(),
            aead_type,
            ss_aead: None,
            ss_len_buf: [0u8; 18],
        }
    }
}

#[async_trait]
impl ProxyWriter for SsStreamWriter {
    async fn write(&mut self, raw_data: &[u8]) -> io::Result<()> {
        // Check if this is the first read. If first read,creat the SsAead.
        if self.ss_aead.is_none() {
            // TODO Creat random slat.
            let mut salt = [0u8; 32];
            let ss_aead = SsAead::new(&salt, self.password.as_slice(), &self.aead_type)
                .or_else(|e| { Err(change_error(e)) })?;
            self.ss_aead = Some(ss_aead)
        }
        let mut aead = self.ss_aead.as_mut().expect("");
        let result = encrypt(raw_data, aead)?;
        self.stream.write_all(result.as_slice()).await
    }
}

fn change_error(error: EncryptError) -> io::Error {
    println!("Stream encrypt error: {}", error);
    io::Error::from(ErrorKind::InvalidInput)
}

fn decrypt(de_data: &[u8], ss_aead: &mut SsAead) -> io::Result<Vec<u8>> {
    match ss_aead.ss_decrypt(de_data) {
        Ok(de_data) => Ok(de_data),
        Err(e) => Err(change_error(e)),
    }
}

fn encrypt(raw_data: &[u8], ss_aead: &mut SsAead) -> io::Result<Vec<u8>> {
    match ss_aead.ss_encrypt(raw_data) {
        Ok(en_data) => Ok(en_data),
        Err(e) => Err(change_error(e)),
    }
}