use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;
use ring::aead::{BoundKey, SealingKey, Aad, OpeningKey, UnboundKey};
use crate::encrypt::Nonce;

#[derive(Copy, Clone, Debug)]
pub enum AeadType {
    AES128GCM,
    AES256GCM,
    Chacha20Poly1305,
}

pub trait AeadEncrypt {
    /// Aead Encrypt
    fn encrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>>;

    /// Aead Decrypt
    fn decrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>>;
}

pub const AEAD_TAG_SIZE: usize = 16;

pub struct AeadEncryptRing {
    sealing_key: SealingKey<Nonce>,
    opening_key: OpeningKey<Nonce>,
}

impl AeadEncryptRing {
    pub fn new(aead_type: &AeadType, key: &[u8]) -> Self {
        let algorithm = match aead_type {
            AeadType::AES128GCM => &ring::aead::AES_128_GCM,
            AeadType::AES256GCM => &ring::aead::AES_256_GCM,
            AeadType::Chacha20Poly1305 => &ring::aead::CHACHA20_POLY1305
        };
        let seal_unbound_key = UnboundKey::new(algorithm, key).unwrap();
        let sealing_key = SealingKey::new(seal_unbound_key, Nonce::new());
        let open_unbound_key = UnboundKey::new(algorithm, key).unwrap();
        let opening_key = OpeningKey::new(open_unbound_key, Nonce::new());
        Self { sealing_key, opening_key }
    }

    pub fn encrypt(&mut self, data: &mut [u8], buffer: &mut [u8]) -> Result<usize> {
        let data_len = data.len();
        self.sealing_key.seal_in_place_separate_tag(Aad::empty(), data).map(|tag| {
            buffer[..data_len].copy_from_slice(data);
            buffer[data_len..data_len + AEAD_TAG_SIZE].copy_from_slice(tag.as_ref());
            data_len + AEAD_TAG_SIZE
        }).or(Err(EncryptError::EncryptErr))
    }

    pub fn decrypt<'a>(&mut self, en_data: &'a mut [u8]) -> Result<&'a mut [u8]> {
        self.opening_key.open_in_place(Aad::empty(), en_data).or(Err(EncryptError::DecryptErr))
    }
}