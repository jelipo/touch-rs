use std::borrow::Borrow;
use std::io;

use async_std::io::ErrorKind;
use async_std::io::ReadExt;
use async_std::net::{Shutdown, TcpStream};
use async_std::prelude::*;
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};

use crate::encrypt::aead::AeadType;
use crate::encrypt::error::EncryptError;
use crate::encrypt::ss::ss_aead::SsAead;
use crate::net::proxy::{Closer, OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::socks::socks5::Socks5;


pub struct SsStreamReader {
    stream: TcpStream,
    password: Vec<u8>,
    aead_type: AeadType,
    ss_aead: Option<SsAead>,
    ss_len_buf: [u8; 18],
}

impl SsStreamReader {
    pub fn new(stream: TcpStream, password: &str, aead_type: AeadType) -> Self {
        SsStreamReader {
            stream,
            password: password.as_bytes().to_vec(),
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
            let aead = read_slat_to_aead(&self.aead_type, &mut self.stream, self.password.as_ref()).await?;
            self.ss_aead = Some(aead)
        }
        let aead = self.ss_aead.as_mut().unwrap();
        //Read bytes and decrypt byte
        self.stream.read_exact(&mut self.ss_len_buf).await?;
        let len_vec = decrypt(&self.ss_len_buf, aead)?;
        let len = u16::from_be_bytes([len_vec[0], len_vec[1]]);
        let mut en_data = vec![0u8; (len + 16) as usize];
        self.stream.read_exact(&mut en_data).await?;
        decrypt(en_data.as_ref(), aead)
    }

    async fn read_adderss(&mut self) -> io::Result<ProxyInfo> {
        unimplemented!()
    }
}

async fn read_slat_to_aead(aead_type: &AeadType, tcpstream: &mut TcpStream, password: &[u8]) -> io::Result<SsAead> {
    let mut salt: Box<[u8]> = match aead_type {
        AeadType::AES128GCM => [0u8; 16].into(),
        AeadType::AES256GCM | AeadType::Chacha20Poly1305 => [0u8; 32].into()
    };
    tcpstream.read_exact(&mut salt).await?;
    SsAead::new(salt.into(), password, aead_type).or_else(|e| { Err(change_error(e)) })
}

pub struct SsStreamWriter {
    stream: TcpStream,
    password: Vec<u8>,
    aead_type: AeadType,
    ss_aead: SsAead,
    ss_len_buf: [u8; 18],
    addr_arr: Option<Box<[u8]>>,
}

impl SsStreamWriter {
    pub fn new(stream: TcpStream, password: &str, aead_type: AeadType, ss_aead: SsAead) -> Self {
        SsStreamWriter {
            stream,
            password: password.as_bytes().to_vec(),
            aead_type,
            ss_aead,
            ss_len_buf: [0u8; 18],
            addr_arr: None,
        }
    }
}

#[async_trait]
impl ProxyWriter for SsStreamWriter {
    async fn write(&mut self, raw_data: &[u8]) -> io::Result<()> {
        let mut aead = &mut self.ss_aead;
        let len = raw_data.len() as u16;
        let len_en = encrypt(&len.to_be_bytes(), aead)?;
        self.stream.write_all(len_en.as_ref()).await?;
        let en_data = encrypt(raw_data, aead)?;
        self.stream.write_all(en_data.as_ref()).await
    }

    async fn write_adderss(&mut self, info: &ProxyInfo) -> io::Result<()> {
        self.stream.write_all(self.ss_aead.salt.borrow()).await?;
        let addr_arr = Socks5::socks5_addr_arr(info.address.as_ref(), info.port, &info.address_type);
        self.write(addr_arr.borrow()).await
    }
}

fn change_error(error: EncryptError) -> io::Error {
    error!("Stream encrypt error: {}", error);
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


pub struct SsOutProxy {
    ss_addr: String,
    ss_port: u16,
    password: String,
    aead_type: AeadType,
}

impl SsOutProxy {
    pub fn new(ss_addr: String, ss_port: u16, password: String, aead_type: &AeadType) -> Self {
        Self {
            ss_addr: ss_addr.to_string(),
            ss_port,
            password: password.to_string(),
            aead_type: (*aead_type).clone(),
        }
    }
}

impl OutputProxy for SsOutProxy {
    fn gen_starter(&mut self, proxy_info: ProxyInfo) -> io::Result<Box<dyn OutProxyStarter + Send>> {
        Ok(Box::new(SsOutProxyStarter {
            proxy_info,
            ss_addr: self.ss_addr.clone(),
            ss_port: self.ss_port,
            password: self.password.clone(),
            aead_type: self.aead_type.clone(),
        }))
    }
}

pub struct SsOutProxyStarter {
    proxy_info: ProxyInfo,
    ss_addr: String,
    ss_port: u16,
    password: String,
    aead_type: AeadType,
}

#[async_trait]
impl OutProxyStarter for SsOutProxyStarter {
    async fn new_connect(&mut self) ->
    io::Result<(Box<dyn ProxyReader + Send>, Box<dyn ProxyWriter + Send>, Box<dyn Closer + Send>)> {
        debug!("new connect");
        let addr = format!("{}:{}", self.ss_addr, self.ss_port);
        let output_stream = TcpStream::connect(addr).await?;
        // Creat a random salt
        let write_salt = gen_random_salt(&self.aead_type);
        let write_ss_aead = SsAead::new(write_salt, self.password.as_bytes(), &self.aead_type)
            .or_else(|e| { Err(change_error(e)) })?;

        let reader = SsStreamReader::new(
            output_stream.clone(), self.password.as_str(), self.aead_type);
        let mut writer = SsStreamWriter::new(
            output_stream.clone(), self.password.as_str(), self.aead_type, write_ss_aead);
        let closer = SsCloser { tcp_stream: output_stream.clone() };
        writer.write_adderss(&self.proxy_info).await?;

        Ok((Box::new(reader), Box::new(writer), Box::new(closer)))
    }
}

pub struct SsCloser {
    tcp_stream: TcpStream
}

impl Closer for SsCloser {
    fn shutdown(&mut self) -> io::Result<()> {
        self.tcp_stream.shutdown(Shutdown::Both)
    }
}


fn gen_random_salt(aead_type: &AeadType) -> Box<[u8]> {
    let array: [u8; 32] = rand::random();
    match aead_type {
        AeadType::AES128GCM => array[0..16].into(),
        AeadType::AES256GCM => array[0..32].into(),
        AeadType::Chacha20Poly1305 => array[0..32].into(),
    }
}