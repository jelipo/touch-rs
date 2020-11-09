
use aes_gcm::{Aes128Gcm, Aes256Gcm};
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::aead::generic_array::GenericArray;
use chacha20poly1305::ChaCha20Poly1305;

use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;

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

//------------------------------------AEAD_AES_256_GCM-----------------------------------------
pub struct AeadAes256Gcm {
    cipher: Aes256Gcm
}

impl AeadAes256Gcm {
    pub fn new(key: &[u8]) -> Self {
        let generic_arr = GenericArray::from_slice(key);
        AeadAes256Gcm { cipher: Aes256Gcm::new(generic_arr) }
    }
}

impl AeadEncrypt for AeadAes256Gcm {
    fn encrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        aead_encrypt(&self.cipher, nonce, data)
    }

    fn decrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        aead_decrypt(&self.cipher, nonce, data)
    }
}

//-------------------------------------AEAD_AES_128_GCM-----------------------------------------
pub struct AeadAes128Gcm {
    cipher: Aes128Gcm
}

impl AeadAes128Gcm {
    pub fn new(key: &[u8]) -> Self {
        let generic_arr = GenericArray::from_slice(key);
        AeadAes128Gcm { cipher: Aes128Gcm::new(generic_arr) }
    }
}

impl AeadEncrypt for AeadAes128Gcm {
    fn encrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        aead_encrypt(&self.cipher, nonce, data)
    }

    fn decrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        aead_decrypt(&self.cipher, nonce, data)
    }
}

//--------------------------------AEAD_CHACHA_20_POLY_1305------------------------------------
pub struct AeadChacha20Poly1305 {
    cipher: ChaCha20Poly1305
}

impl AeadChacha20Poly1305 {
    pub fn new(key: &[u8]) -> Self {
        let generic_arr = GenericArray::from_slice(key);
        AeadChacha20Poly1305 { cipher: ChaCha20Poly1305::new(generic_arr) }
    }
}

impl AeadEncrypt for AeadChacha20Poly1305 {
    fn encrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        aead_encrypt(&self.cipher, nonce, data)
    }

    fn decrypt(&self, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        aead_decrypt(&self.cipher, nonce, data)
    }
}

//--------------------------------------------------------------------------------------------

/// Aead Encrypt
fn aead_encrypt<T: Aead>(cipher: &T, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let nonce = GenericArray::from_slice(nonce);
    cipher.encrypt(nonce, data).or(Err(EncryptError::EncryptErr))
}

/// Aead Decrypt
fn aead_decrypt<T: Aead>(cipher: &T, nonce: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let nonce = GenericArray::from_slice(nonce);
    cipher.decrypt(nonce, data).or(Err(EncryptError::DecryptErr))
}

//--------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::encrypt::aead::{AeadAes128Gcm, AeadAes256Gcm, AeadChacha20Poly1305, AeadEncrypt, EncryptError};

    const NONCE: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    #[test]
    fn aes256gcm_test() -> Result<(), EncryptError> {
        let plain_text = b"hello";
        let salt = [0u8; 32];
        let master_key = plain_text;
        let subkey = [0u8; 32];
        let aes256gcm = AeadAes256Gcm::new(&subkey);
        let en_vec = aes256gcm.encrypt(&NONCE, plain_text)?;
        let de_vec = aes256gcm.decrypt(&NONCE, en_vec.as_slice())?;
        assert_eq!(plain_text.to_vec(), de_vec);
        Ok(())
    }

    #[test]
    fn aes128gcm_test() -> Result<(), EncryptError> {
        let plain_text = b"hello";
        let salt = [0u8; 16];
        let master_key = plain_text;
        let subkey = [0u8; 16];
        let aes128gcm = AeadAes128Gcm::new(&subkey);
        let en_vec = aes128gcm.encrypt(&NONCE, plain_text)?;
        let de_vec = aes128gcm.decrypt(&NONCE, en_vec.as_slice())?;
        assert_eq!(plain_text.to_vec(), de_vec);
        Ok(())
    }

    #[test]
    fn chacha20poly1305_test() -> Result<(), EncryptError> {
        let plain_text = b"hello";
        let salt = [0u8; 32];
        let master_key = plain_text;
        let subkey = [0u8; 32];
        let chacha20poly1305 = AeadChacha20Poly1305::new(&subkey);
        let en_vec = chacha20poly1305.encrypt(&NONCE, plain_text)?;
        let de_vec = chacha20poly1305.decrypt(&NONCE, en_vec.as_slice())?;
        assert_eq!(plain_text.to_vec(), de_vec);
        Ok(())
    }
}