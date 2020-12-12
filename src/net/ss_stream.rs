use std::borrow::Borrow;
use std::convert::TryFrom;
use std::io;
use std::io::Error;
use std::str::FromStr;

use async_std::io::ErrorKind;
use async_std::io::ReadExt;
use async_std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use async_std::prelude::*;
use async_trait::async_trait;
use log::{debug, error, info};

use crate::core::profile::BasePassiveConfig;
use crate::encrypt::aead::AeadType;
use crate::encrypt::error::EncryptError;
use crate::encrypt::ss::ss_aead::SsAead;
use crate::net::proxy::{Closer, InputProxy, OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::socks::socks5::Socks5;

pub struct SsStreamReader {
    stream: TcpStream,
    password: Vec<u8>,
    aead_type: AeadType,
    ss_aead: Option<SsAead>,
    ss_len_buf: [u8; 18],
    ss_data_buf: Box<[u8]>,
}

impl SsStreamReader {
    pub fn new(stream: TcpStream, password: &str, aead_type: AeadType) -> Self {
        SsStreamReader {
            stream,
            password: password.as_bytes().to_vec(),
            aead_type,
            ss_aead: None,
            ss_len_buf: [0u8; 18],
            ss_data_buf: vec![0u8; 4096].into_boxed_slice(),
        }
    }
}

/// Shadowsocks TCP Reader.
/// First, it will read a 16/32 bytes of salt.
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
        let len: usize = (((0x0000 | len_vec[0]) << 8) | len_vec[1]) as usize;
        let buf = self.ss_data_buf[..(len + 16) as usize].as_mut();
        self.stream.read_exact(buf).await?;
        decrypt(buf, aead)
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
    ss_aead: SsAead,
    addr_arr: Option<Box<[u8]>>,
}

impl SsStreamWriter {
    pub fn new(stream: TcpStream, ss_aead: SsAead) -> Self {
        SsStreamWriter {
            stream,
            ss_aead,
            addr_arr: None,
        }
    }
}

#[async_trait]
impl ProxyWriter for SsStreamWriter {
    async fn write(&mut self, raw_data: &[u8]) -> io::Result<()> {
        let aead = &mut self.ss_aead;
        let len = raw_data.len() as u16;
        let len_en = encrypt(&len.to_be_bytes(), aead)?;
        self.stream.write_all(len_en.as_ref()).await?;
        let en_data = encrypt(raw_data, aead)?;
        self.stream.write_all(en_data.as_ref()).await
    }

    async fn write_adderss(&mut self, info: &ProxyInfo) -> io::Result<()> {
        self.stream.write_all(self.ss_aead.salt.borrow()).await?;
        let addr_arr = Socks5::socks5_addr_arr(&info.address, info.port, &info.address_type);
        self.write(&addr_arr).await
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

//------------------------------SS_OUT_PROXY-----------------------------------------
#[derive(Clone)]
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
    fn gen_starter(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        Ok(Box::new(SsOutProxyStarter {
            ss_addr: self.ss_addr.clone(),
            ss_port: self.ss_port,
            password: self.password.clone(),
            aead_type: self.aead_type.clone(),
        }))
    }
}

pub struct SsOutProxyStarter {
    ss_addr: String,
    ss_port: u16,
    password: String,
    aead_type: AeadType,
}

#[async_trait]
impl OutProxyStarter for SsOutProxyStarter {
    async fn new_connect(&mut self, proxy_info: ProxyInfo) ->
    io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>, Box<dyn Closer>)> {
        debug!("new connect");
        let addr = format!("{}:{}", self.ss_addr, self.ss_port);
        let output_stream = TcpStream::connect(addr).await?;
        // Creat a random salt
        let write_salt = gen_random_salt(&self.aead_type);
        let write_ss_aead = SsAead::new(write_salt, self.password.as_bytes(), &self.aead_type)
            .or_else(|e| { Err(change_error(e)) })?;

        let reader = SsStreamReader::new(
            output_stream.clone(), self.password.as_str(), self.aead_type);
        let mut writer = SsStreamWriter::new(output_stream.clone(), write_ss_aead);
        let closer = SsCloser { tcp_stream: output_stream.clone() };
        writer.write_adderss(&proxy_info).await?;

        Ok((Box::new(reader), Box::new(writer), Box::new(closer)))
    }
}
//<--<--<--<--<--<--<--<--<--<--<--<--SS_OUT_PROXY--<--<--<--<--<--<--<--<--<--<--<--<

//>-->-->-->-->-->-->-->-->-->-->-->--SS_CLOSER-->-->-->-->-->-->-->-->-->-->-->-->
pub struct SsCloser {
    tcp_stream: TcpStream
}

impl Closer for SsCloser {
    fn shutdown(&mut self) -> io::Result<()> {
        self.tcp_stream.shutdown(Shutdown::Both)
    }
}
//<--<--<--<--<--<--<--<--<--<--<--<--SS_CLOSER--<--<--<--<--<--<--<--<--<--<--<--<


//>-->-->-->-->-->-->-->-->-->-->-->--SS_INPUT_PROXY-->-->-->-->-->-->-->-->-->-->-->-->

pub struct SsInputProxy {
    tcp_listener: TcpListener,
    password: String,
    out_proxy: Box<dyn OutputProxy>,
    aead_type: AeadType,
}

impl SsInputProxy {
    pub async fn new(
        aead_type: AeadType,
        passive: &BasePassiveConfig,
        out_proxy: Box<dyn OutputProxy>,
    ) -> io::Result<Self> {
        let addr_str = format!("{}:{}", &passive.local_host, passive.local_port);
        let addr = SocketAddr::from_str(addr_str.as_str()).or(
            Err(Error::new(ErrorKind::InvalidInput, "Error address"))
        );
        let tcp_listener = TcpListener::bind(addr?).await?;
        info!("Shadowsocks ({:?}) bind in {}", aead_type, addr_str);
        let password = passive.password.clone()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Shadowsocks must have a password"))?;
        Ok(Self {
            tcp_listener,
            password,
            out_proxy,
            aead_type,
        })
    }
}

#[async_trait]
impl InputProxy for SsInputProxy {
    async fn start(&mut self) -> io::Result<()> {
        info!("Shadowsocks start listen");
        loop {
            let tcpstream: TcpStream = self.tcp_listener.incoming().next().await.ok_or(
                io::Error::new(ErrorKind::InvalidInput, "")
            )??;
            let starter = match self.out_proxy.gen_starter() {
                Ok(n) => n,
                Err(_) => continue
            };
            let aead_type = self.aead_type.clone();
            let password = self.password.clone();
            async_std::task::spawn(async move {
                if let Err(e) = new_ss_proxy(tcpstream, starter, aead_type, password).await {
                    error!("Shadowsocks input proxy error. {}", e)
                };
            });
        }
    }
}

async fn new_ss_proxy(input: TcpStream, mut starter: Box<dyn OutProxyStarter>,
                      aead_type: AeadType, password: String) -> io::Result<()> {
    let mut ss_reader = SsStreamReader::new(input.clone(), password.as_str(), aead_type.clone());
    let write_slat = gen_random_salt(&aead_type);
    let write_aead = SsAead::new(write_slat, password.as_bytes(), &aead_type).or_else(
        |e| { Err(change_error(e)) })?;
    let ss_writer = SsStreamWriter::new(input.clone(), write_aead);

    let first_read_data = ss_reader.read().await?;
    let (info, read_addr_size) = Socks5::read_to_socket_addrs(&first_read_data);
    let (out_reader,
        out_writer,
        mut closer) = starter.new_connect(info).await?;

    let reader = async {
        ss_input_write(ss_writer, out_reader).await
    };

    let writer = async {
        let first_write = if first_read_data.len() == read_addr_size { None } else {
            Some(first_read_data[read_addr_size..].to_vec())
        };
        ss_input_read(ss_reader, out_writer, first_write).await
    };
    // Wait for two futures done.
    let _size = reader.race(writer).await;
    let _sd_rs = input.shutdown(Shutdown::Both);
    let _closer_rs = closer.shutdown();
    Ok(())
}

async fn ss_input_read(
    mut ss_reader: SsStreamReader, mut out_writer: Box<dyn ProxyWriter>, first_write: Option<Vec<u8>>,
) -> usize {
    let mut total = 0;
    if let Some(data) = first_write {
        if out_writer.write(&data).await.is_err() { return 0; } else { total = data.len() }
    }
    while let Ok(data) = ss_reader.read().await {
        let size = data.len();
        if size == 0 { break; } else { total = total + size; }
        if out_writer.write(&data).await.is_err() { break; }
    }
    total
}

async fn ss_input_write(mut input_write: SsStreamWriter, mut out_reader: Box<dyn ProxyReader>) -> usize {
    let mut total = 0;
    while let Ok(data) = out_reader.read().await {
        let size = data.len();
        if size == 0 { break; } else { total = total + size; }
        if input_write.write(&data).await.is_err() { break; };
    }
    total
}

//<--<--<--<--<--<--<--<--<--<--<--<--SS_INPUT_PROXY--<--<--<--<--<--<--<--<--<--<--<--<

/// Generate a Shadowsocks salt
fn gen_random_salt(aead_type: &AeadType) -> Box<[u8]> {
    match aead_type {
        AeadType::AES128GCM => rand::random::<[u8; 16]>().into(),
        AeadType::AES256GCM | AeadType::Chacha20Poly1305 => rand::random::<[u8; 32]>().into()
    }
}