use std::borrow::Borrow;

use crate::encrypt::aead::{AeadEncrypt, AeadType, AeadEncryptRing, AEAD_TAG_SIZE};
use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;
use crate::encrypt::ss::{generate_16_sub_key, generate_32_sub_key, openssl_bytes_to_key, generate_subkey};


pub struct SsAead {
    encryption: AeadEncryptRing,
    pub salt: Box<[u8]>,
    buffer: Box<[u8]>,
}

/// Shadowsocks protocol supporting AEAD encryption
impl SsAead {
    /// Initialize according to the specified enum
    ///
    /// * `salt` - 16/32 bytes of each TCP connection header
    /// * `password` - User's simple password
    /// * `aead_type` - Aead type
    pub fn new(salt: Box<[u8]>, password: &[u8], aead_type: &AeadType) -> Result<Self> {
        let mut master_key = [0u8; 32];
        let len = match aead_type {
            AeadType::AES128GCM => 16,
            AeadType::AES256GCM | AeadType::Chacha20Poly1305 => 32
        };
        openssl_bytes_to_key(password, &mut master_key[..len]);
        let aead_key = generate_subkey(salt.borrow(), &mut master_key[..len], len)?;
        let encryption = AeadEncryptRing::new(aead_type, aead_key.as_ref());
        Ok(SsAead { encryption, salt, buffer: vec![0u8; 32 * 1024].into_boxed_slice() })
    }

    pub fn ss_encrypt(&mut self, data: &mut [u8]) -> Result<&mut [u8]> {
        if self.buffer.len() < data.len() + AEAD_TAG_SIZE {
            self.buffer = vec![0u8; data.len() + AEAD_TAG_SIZE].into_boxed_slice()
        }
        let size = self.encryption.encrypt(data, self.buffer.as_mut())?;
        Ok(&mut self.buffer[..size])
    }

    pub fn ss_decrypt<'a>(&mut self, en_data: &'a mut [u8]) -> Result<&'a mut [u8]> {
        self.encryption.decrypt(en_data)
    }
}