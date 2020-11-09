use std::borrow::BorrowMut;
use std::io;
use std::io::ErrorKind;

use async_std::io::Read;
use async_std::io::ReadExt;
use async_std::stream::Stream;
use async_trait::async_trait;

use crate::encrypt::aead::AeadType;
use crate::encrypt::error::EncryptError;
use crate::encrypt::ss::ss_aead::SsAead;

#[async_trait(? Send)]
pub trait StreamReader {
    async fn read(&mut self) -> io::Result<Vec<u8>>;
}

pub struct SsStreamReader<'a> {
    stream: Box<&'a mut (dyn Read + Unpin)>,
    password: &'a [u8],
    aead_type: &'a AeadType,
    ss_aead: Option<SsAead>,
    ss_len_buf: [u8; 18],
}

impl<'a> SsStreamReader<'a> {
    pub fn new<R>(stream: &'a mut R, password: &'a [u8], aead_type: &'a AeadType) -> Self
        where R: Read + Unpin + Sized {
        SsStreamReader {
            stream: Box::new(stream),
            password,
            aead_type,
            ss_aead: None,
            ss_len_buf: [0u8; 18],
        }
    }
}

#[async_trait(? Send)]
impl StreamReader for SsStreamReader<'_> {
    async fn read(&mut self) -> io::Result<Vec<u8>> {
        // Check if this is the first read. If first read,creat the SsAead.
        if self.ss_aead.is_none() {
            let mut salt = [0u8; 32];
            self.stream.read_exact(&mut salt).await?;
            let ss_aead = match SsAead::new(&salt, self.password, self.aead_type) {
                Ok(ss_aead) => Ok(ss_aead),
                Err(e) => Err(change_error(e)),
            }?;
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
