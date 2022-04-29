use std::borrow::Borrow;
use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::str::FromStr;

use async_trait::async_trait;
use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};

use crate::core::profile::BasePassiveConfig;
use crate::encrypt::aead::AeadType;
use crate::encrypt::error::EncryptError;
use crate::encrypt::ss::ss_aead::SsAead;
use crate::net::proxy::{InputProxy, OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};
use crate::socks::socks5::Socks5;

pub struct SsStreamReader {
    read_half: OwnedReadHalf,
    password: Vec<u8>,
    aead_type: AeadType,
    ss_aead: Option<SsAead>,
    ss_len_buf: [u8; 18],
    ss_data_buf: Vec<u8>,
}

impl SsStreamReader {
    pub fn new(read_half: OwnedReadHalf, password: &str, aead_type: AeadType) -> Self {
        SsStreamReader {
            read_half,
            password: password.as_bytes().to_vec(),
            aead_type,
            ss_aead: None,
            ss_len_buf: [0u8; 18],
            ss_data_buf: vec![0u8; 1024 * 32],
        }
    }
}

/// Shadowsocks TCP Reader.
/// First, it will read a 16/32 bytes of salt.
#[async_trait]
impl ProxyReader for SsStreamReader {
    async fn read(&mut self) -> io::Result<&mut [u8]> {
        // Check if this is the first read. If first read,creat the SsAead.
        if self.ss_aead.is_none() {
            let aead = read_slat_to_aead(&self.aead_type, &mut self.read_half, self.password.as_ref()).await?;
            self.ss_aead = Some(aead)
        }
        let aead = self.ss_aead.as_mut().unwrap();
        //Read bytes and decrypt byte
        self.read_half.read_exact(&mut self.ss_len_buf).await?;
        let len_vec = decrypt(&mut self.ss_len_buf, aead)?;
        let en_data_len = u16::from_be_bytes([len_vec[0], len_vec[1]]) as usize;
        // Automatic capacity expansion
        if en_data_len > self.ss_data_buf.len() {
            self.ss_data_buf = vec![0u8; en_data_len]
        }
        let buf = self.ss_data_buf[..(en_data_len + 16) as usize].as_mut();
        self.read_half.read_exact(buf).await?;
        decrypt(buf, aead)
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Read slat from TCP , and initialize a Shadowsocks AEAD.
async fn read_slat_to_aead(aead_type: &AeadType, readhalf: &mut OwnedReadHalf, password: &[u8]) -> io::Result<SsAead> {
    let mut salt = match aead_type {
        AeadType::AES128GCM => vec![0u8; 16],
        AeadType::AES256GCM | AeadType::Chacha20Poly1305 => vec![0u8; 32],
    };
    readhalf.read_exact(&mut salt).await?;
    SsAead::new(salt, password, aead_type).map_err(change_error)
}

pub struct SsStreamWriter {
    writehalf: OwnedWriteHalf,
    ss_aead: SsAead,
    proxy_info: Option<ProxyInfo>,
}

impl SsStreamWriter {
    /// Create a pure Shadowsocks writer.
    /// It will only faithfully send the en_data you want to transmit,
    /// and will not automatically send the ss_header.
    pub fn creat_without_info(writehalf: OwnedWriteHalf, ss_aead: SsAead) -> Self {
        SsStreamWriter {
            writehalf,
            ss_aead,
            proxy_info: None,
        }
    }

    /// Creat a new [SsStreamWriter] with [ProxyInfo], and this writer will send
    /// a bytes of ss_header when you first write.
    pub fn new_with_addr(writehalf: OwnedWriteHalf, ss_aead: SsAead, proxy_info: ProxyInfo) -> Self {
        SsStreamWriter {
            writehalf,
            ss_aead,
            proxy_info: Some(proxy_info),
        }
    }

    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.writehalf.shutdown().await
    }

    async fn en_write(&mut self, raw_data: &mut [u8]) -> io::Result<()> {
        let aead = &mut self.ss_aead;
        let len = raw_data.len() as u16;
        let len_en = encrypt(&mut len.to_be_bytes(), aead)?;
        self.writehalf.write_all(len_en.as_ref()).await?;
        let en_data = encrypt(raw_data, aead)?;
        self.writehalf.write_all(en_data.as_ref()).await
    }
}

#[async_trait]
impl ProxyWriter for SsStreamWriter {
    async fn write(&mut self, raw_data: &mut [u8]) -> io::Result<()> {
        if let Some(info) = &self.proxy_info {
            self.writehalf.write_all(self.ss_aead.salt.borrow()).await?;
            let mut addr_arr = Socks5::socks5_addr_arr(&info.address, info.port, &info.address_type);
            self.en_write(&mut addr_arr).await?;
            self.proxy_info = None;
        }
        self.en_write(raw_data).await
    }

    async fn shutdown(&mut self) -> io::Result<()> {
        self.writehalf.shutdown().await
    }
}

fn change_error(error: EncryptError) -> io::Error {
    error!("Stream encrypt error: {}", error);
    io::Error::from(ErrorKind::InvalidInput)
}

fn decrypt<'a>(de_data: &'a mut [u8], ss_aead: &'a mut SsAead) -> io::Result<&'a mut [u8]> {
    match ss_aead.ss_decrypt(de_data) {
        Ok(de_data) => Ok(de_data),
        Err(e) => Err(change_error(e)),
    }
}

fn encrypt<'a>(raw_data: &mut [u8], ss_aead: &'a mut SsAead) -> io::Result<&'a mut [u8]> {
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
            ss_addr,
            ss_port,
            password,
            aead_type: (*aead_type),
        }
    }
}

impl OutputProxy for SsOutProxy {
    fn gen_connector(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        Ok(Box::new(SsOutProxyStarter {
            ss_addr: self.ss_addr.clone(),
            ss_port: self.ss_port,
            password: self.password.clone(),
            aead_type: self.aead_type,
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
    async fn new_connection(&mut self, proxy_info: ProxyInfo) -> io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>)> {
        debug!("new connect");
        let addr = format!("{}:{}", self.ss_addr, self.ss_port);
        let output_stream = TcpStream::connect(addr).await?;
        // Creat a random salt
        let write_salt = gen_random_salt(&self.aead_type);
        let write_ss_aead = SsAead::new(write_salt, self.password.as_bytes(), &self.aead_type).map_err(change_error)?;
        let (read_half, write_half) = output_stream.into_split();

        let reader = SsStreamReader::new(read_half, self.password.as_str(), self.aead_type);
        let writer = SsStreamWriter::new_with_addr(write_half, write_ss_aead, proxy_info);
        Ok((Box::new(reader), Box::new(writer)))
    }
}
//<--<--<--<--<--<--<--<--<--<--<--<--SS_OUT_PROXY--<--<--<--<--<--<--<--<--<--<--<--<

//>-->-->-->-->-->-->-->-->-->-->-->--SS_INPUT_PROXY-->-->-->-->-->-->-->-->-->-->-->-->

pub struct SsInputProxy {
    tcp_listener: TcpListener,
    password: String,
    out_proxy: Box<dyn OutputProxy>,
    aead_type: AeadType,
}

impl SsInputProxy {
    pub async fn new(aead_type: AeadType, passive: &BasePassiveConfig, out_proxy: Box<dyn OutputProxy>) -> io::Result<Self> {
        let addr_str = format!("{}:{}", &passive.local_host, passive.local_port);
        let addr = SocketAddr::from_str(addr_str.as_str()).map_err(|_| Error::new(ErrorKind::InvalidInput, "Error address"));
        let tcp_listener = TcpListener::bind(addr?).await?;
        info!("Shadowsocks ({:?}) bind in {}", aead_type, addr_str);
        let password =
            passive.password.clone().ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Shadowsocks must have a password"))?;
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
            let (tcpstream, _addr) = self.tcp_listener.accept().await?;
            let starter = match self.out_proxy.gen_connector() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let aead_type = self.aead_type;
            let password = self.password.clone();
            tokio::task::spawn(async move {
                if let Err(e) = new_ss_proxy(tcpstream, starter, aead_type, password).await {
                    error!("Shadowsocks input proxy error. {}", e)
                };
            });
        }
    }
}

async fn new_ss_proxy(
    tcpstream: TcpStream,
    mut starter: Box<dyn OutProxyStarter>,
    aead_type: AeadType,
    password: String,
) -> io::Result<()> {
    let (read_half, write_half) = tcpstream.into_split();
    let mut ss_reader = SsStreamReader::new(read_half, password.as_str(), aead_type);
    let write_slat = gen_random_salt(&aead_type);
    let write_aead = SsAead::new(write_slat, password.as_bytes(), &aead_type).map_err(change_error)?;
    let ss_writer = SsStreamWriter::creat_without_info(write_half, write_aead);

    let first_read_data = ss_reader.read().await?;
    let (info, read_addr_size) = Socks5::read_to_socket_addrs(first_read_data);
    let (mut out_reader, mut out_writer) = starter.new_connection(info).await?;

    let reader = ss_input_write(ss_writer, &mut *out_reader);
    let size = first_read_data.len();
    let first_write: Option<Box<[u8]>> = if size == read_addr_size {
        None
    } else {
        Some(first_read_data[read_addr_size..].into())
    };
    let writer = ss_input_read(ss_reader, &mut *out_writer, first_write);
    // Wait for two futures done.
    tokio::select! {
        _ = reader => {}
        _ = writer => {}
    }
    // TODO Dont know TCP will be dropped.
    // let _sd_rs = input.shutdown().await;
    Ok(())
}

async fn ss_input_read(mut ss_reader: SsStreamReader, out_writer: &mut dyn ProxyWriter, first_write: Option<Box<[u8]>>) -> usize {
    let mut total = 0;
    if let Some(mut data) = first_write {
        if out_writer.write(data.as_mut()).await.is_err() {
            return 0;
        } else {
            total = data.len()
        }
    }
    while let Ok(data) = ss_reader.read().await {
        let size = data.len();
        if size == 0 {
            break;
        } else {
            total += size;
        }
        if out_writer.write(data.as_mut()).await.is_err() {
            break;
        }
    }
    let _read_result = ss_reader.shutdown().await;
    let _write_result = out_writer.shutdown().await;
    total
}

async fn ss_input_write(mut input_write: SsStreamWriter, out_reader: &mut dyn ProxyReader) -> usize {
    let mut total = 0;
    while let Ok(data) = out_reader.read().await {
        let size = data.len();
        if size == 0 {
            break;
        } else {
            total += size;
        }
        if input_write.write(data.as_mut()).await.is_err() {
            break;
        };
    }
    let _read_result = input_write.shutdown().await;
    let _write_result = out_reader.shutdown().await;
    total
}

//<--<--<--<--<--<--<--<--<--<--<--<--SS_INPUT_PROXY--<--<--<--<--<--<--<--<--<--<--<--<

/// Generate a Shadowsocks salt
fn gen_random_salt(aead_type: &AeadType) -> Vec<u8> {
    match aead_type {
        AeadType::AES128GCM => rand::random::<[u8; 16]>().into(),
        AeadType::AES256GCM | AeadType::Chacha20Poly1305 => rand::random::<[u8; 32]>().into(),
    }
}
