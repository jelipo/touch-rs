use hkdf::Hkdf;
use sha1::Sha1;

use crate::encrypt::aead::{AeadAes256Gcm};
use crate::encrypt::ss::ss_aead::SsAead;
use crate::encrypt::error::EncryptError;
use crate::encrypt::error::EncryptError::InvalidSaltSize;

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";
pub const AES_256_SZIE: usize = 32;
pub const AES_128_SZIE: usize = 16;

impl SsAead<[u8; AES_256_SZIE]> for AeadAes256Gcm {
    ///
    fn generate_sub_key(salt_arr: &[u8], master_key: &[u8]) -> Result<[u8; AES_256_SZIE], EncryptError> {
        if salt_arr.len() != AES_256_SZIE { return Err(InvalidSaltSize(AES_256_SZIE as u8)); }
        let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
        let mut subkey = [0u8; AES_256_SZIE];
        hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
        return Ok(subkey);
    }
}