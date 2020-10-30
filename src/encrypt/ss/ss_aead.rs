use hkdf::Hkdf;
use sha1::Sha1;

use crate::encrypt::error::EncryptError;
use crate::encrypt::error::Result;
use crate::encrypt::ss::ss_aead::EncryptError::InvalidSaltSize;


pub const AES_256_SZIE: usize = 32;
pub const AES_128_SZIE: usize = 16;

/// Shadowsocks AEAD trait
trait SsAead {
    fn new(password: &[u8]) -> Result<Vec<u8>>;

    fn ss_encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    fn ss_decrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
}

pub struct SsAes256Gcm {}

pub struct SsAes128Gcm {}

pub struct SsChacha20Poly1305 {}






