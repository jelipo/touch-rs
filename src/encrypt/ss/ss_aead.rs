use std::borrow::Borrow;

use crate::encrypt::aead::{AeadEncrypt, AeadType, AeadEncryptRing, AEAD_TAG_SIZE};
use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;
use crate::encrypt::ss::{gen_master_key, generate_16_sub_key, generate_32_sub_key};




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
        let master_key = gen_master_key(password);
        let aead_key: Box<[u8]> = match aead_type {
            AeadType::AES128GCM => generate_16_sub_key(salt.borrow(), &master_key)?.into(),
            AeadType::AES256GCM |
            AeadType::Chacha20Poly1305 => generate_32_sub_key(salt.borrow(), &master_key)?.into(),
            _ => return Err(EncryptError::NotSupport)
        };
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


#[cfg(test)]
mod tests {


    //
    // #[test]
    // fn ss_aes256gcm_test() {
    //     let slat = [0u8; 32];
    //     let mut ss_aead = SsAead::new(slat.into(), b"test", &AeadType::AES256GCM).unwrap();
    //     let de_data: [u8; 2] = (1024 as u16).to_be_bytes();
    //     let en_data = Box::new(ss_aead.ss_encrypt(&de_data).unwrap());
    //     let de_data2 = Box::new(ss_aead.ss_decrypt(en_data.as_slice()).unwrap());
    //     assert_eq!(&de_data, de_data2.as_slice());
    // }
    //
    // #[test]
    // fn ss_aes128gcm_test() {
    //     let slat = [0u8; 16];
    //     let mut ss_aead = SsAead::new(slat.into(), b"test", &AeadType::AES128GCM).unwrap();
    //     let de_data: [u8; 2] = (1024 as u16).to_be_bytes();
    //     let en_data = Box::new(ss_aead.ss_encrypt(&de_data).unwrap());
    //     let de_data2 = Box::new(ss_aead.ss_decrypt(en_data.as_slice()).unwrap());
    //     assert_eq!(&de_data, de_data2.as_slice());
    // }
    //
    // #[test]
    // fn ss_chacha20poly1305_test() {
    //     let slat = [0u8; 32];
    //     let mut ss_aead = SsAead::new(slat.into(), b"test", &AeadType::Chacha20Poly1305).unwrap();
    //     let de_data: [u8; 2] = (1024 as u16).to_be_bytes();
    //     let en_data = Box::new(ss_aead.ss_encrypt(&de_data).unwrap());
    //     let de_data2 = Box::new(ss_aead.ss_decrypt(en_data.as_slice()).unwrap());
    //     assert_eq!(&de_data, de_data2.as_slice());
    // }
}