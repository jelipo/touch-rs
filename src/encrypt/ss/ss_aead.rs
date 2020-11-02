use chacha20poly1305::aead::NewAead;
use chacha20poly1305::ChaCha20Poly1305;
use hkdf::Hkdf;
use sha1::Sha1;

use crate::encrypt::aead::{AeadAes128Gcm, AeadAes256Gcm, AeadChacha20Poly1305, AeadEncrypt, AeadType};
use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;
use crate::encrypt::ss::{gen_master_key, generate_16_sub_key, generate_32_sub_key};
use crate::encrypt::ss::ss_aead::EncryptError::InvalidSaltSize;

pub struct SsAead {
    encryption: Box<dyn AeadEncrypt>
}

impl SsAead {
    fn new(salt: &[u8], password: &[u8], aead_type: &AeadType) -> Result<Self> {
        let master_key = gen_master_key(password);
        let encryption: Box<dyn AeadEncrypt> = match aead_type {
            AeadType::AES128GCM => {
                if 16 != salt.len() { return Err(EncryptError::InvalidSaltSize(16)); }
                let subkey = generate_16_sub_key(salt, &master_key)?;
                Box::new(AeadAes128Gcm::new(&subkey))
            }
            AeadType::AES256GCM => {
                if 32 != salt.len() { return Err(EncryptError::InvalidSaltSize(32)); }
                let subkey = generate_32_sub_key(salt, &master_key)?;
                Box::new(AeadAes256Gcm::new(&subkey))
            }
            AeadType::Chacha20Poly1305 => {
                if 32 != salt.len() { return Err(EncryptError::InvalidSaltSize(32)); }
                let subkey = generate_32_sub_key(salt, &master_key)?;
                Box::new(AeadChacha20Poly1305::new(&subkey))
            }
        };
        Ok(SsAead { encryption })
    }

    fn ss_encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.encryption.encrypt(&[0; 12], data)
    }

    fn ss_decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.encryption.decrypt(&[0; 12], data)
    }
}

pub struct SsAes256Gcm {}


pub struct SsAes128Gcm {}

pub struct SsChacha20Poly1305 {}






