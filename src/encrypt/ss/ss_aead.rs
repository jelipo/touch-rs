use std::borrow::Borrow;

use crate::encrypt::aead::{AeadAes128Gcm, AeadAes256Gcm, AeadChacha20Poly1305, AeadEncrypt, AeadType};
use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;
use crate::encrypt::ss::{gen_master_key, generate_16_sub_key, generate_32_sub_key};

pub struct SsAead {
    encryption: Box<dyn AeadEncrypt + Send>,
    en_nonce: Nonce,
    de_nonce: Nonce,
}

/// Shadowsocks protocol supporting AEAD encryption
impl SsAead {
    /// Initialize according to the specified enum
    ///
    /// * `salt` - 16/32 bytes of each TCP connection header
    /// * `password` - User's simple password
    /// * `aead_type` - Aead type
    pub fn new(salt: &[u8], password: &[u8], aead_type: &AeadType) -> Result<Self> {
        let master_key = gen_master_key(password);
        let encryption: Box<dyn AeadEncrypt + Send> = match aead_type {
            AeadType::AES128GCM => {
                let subkey = generate_16_sub_key(salt, &master_key)?;
                Box::new(AeadAes128Gcm::new(&subkey))
            }
            AeadType::AES256GCM => {
                let subkey = generate_32_sub_key(salt, &master_key)?;
                Box::new(AeadAes256Gcm::new(&subkey))
            }
            AeadType::Chacha20Poly1305 => {
                let subkey = generate_32_sub_key(salt, &master_key)?;
                Box::new(AeadChacha20Poly1305::new(&subkey))
            }
            _ => return Err(EncryptError::NotSupport)
        };
        Ok(SsAead {
            encryption,
            de_nonce: Nonce::new(),
            en_nonce: Nonce::new(),
        })
    }

    pub fn ss_encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce_arr = self.en_nonce.get_and_increment();
        self.encryption.encrypt(nonce_arr.borrow(), data)
    }

    pub fn ss_decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let nonce_arr = self.de_nonce.get_and_increment();
        self.encryption.decrypt(nonce_arr.borrow(), data)
    }
}

#[derive(Clone)]
pub struct Nonce {
    base: u128
}

impl Nonce {
    /// Get the nonce byte and increment
    pub fn get_and_increment(&mut self) -> Box<[u8]> {
        let byt: [u8; 16] = self.base.to_le_bytes();
        self.base = self.base + 1;
        byt[0..12].into()
    }

    pub fn new() -> Self { Nonce { base: 0 } }
}

#[cfg(test)]
mod tests {
    use crate::core::profile::ProtocalType;
    use crate::encrypt::aead::AeadType;
    use crate::encrypt::ss::ss_aead::SsAead;

    #[test]
    fn ss_aes256gcm_test() {
        let slat = [0u8; 32];
        let mut ss_aead = SsAead::new(&slat, b"test",
                                      &AeadType::AES256GCM).unwrap();
        let de_data: [u8; 2] = (1024 as u16).to_be_bytes();
        let en_data = Box::new(ss_aead.ss_encrypt(&de_data).unwrap());
        let de_data2 = Box::new(ss_aead.ss_decrypt(en_data.as_slice()).unwrap());
        assert_eq!(&de_data, de_data2.as_slice());
    }

    #[test]
    fn ss_aes128gcm_test() {
        let slat = [0u8; 16];
        let mut ss_aead = SsAead::new(&slat, b"test",
                                      &AeadType::AES128GCM).unwrap();
        let de_data: [u8; 2] = (1024 as u16).to_be_bytes();
        let en_data = Box::new(ss_aead.ss_encrypt(&de_data).unwrap());
        let de_data2 = Box::new(ss_aead.ss_decrypt(en_data.as_slice()).unwrap());
        assert_eq!(&de_data, de_data2.as_slice());
    }

    #[test]
    fn ss_chacha20poly1305_test() {
        let slat = [0u8; 32];
        let mut ss_aead = SsAead::new(&slat, b"test",
                                      &AeadType::Chacha20Poly1305).unwrap();
        let de_data: [u8; 2] = (1024 as u16).to_be_bytes();
        let en_data = Box::new(ss_aead.ss_encrypt(&de_data).unwrap());
        let de_data2 = Box::new(ss_aead.ss_decrypt(en_data.as_slice()).unwrap());
        assert_eq!(&de_data, de_data2.as_slice());
    }
}