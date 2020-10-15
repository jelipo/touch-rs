use aes_gcm::aead::{Aead, generic_array::GenericArray, NewAead};
use aes_gcm::Aes256Gcm;
use hkdf::Hkdf;
use sha1::Sha1;

use crate::encrypt::ss_aead::{AeadError, SsAead};
use crate::encrypt::ss_aead::AeadError::InvalidSaltSize;
use crate::encrypt::ss_aead::Result;

pub struct AeadAesGcm {}

impl AeadAesGcm {
    /// 加密
    pub fn encrypt(&self) {
        let key_arr = b"an example very very secret key.";
        let key = GenericArray::from_slice(key_arr);

        let cipher = Aes256Gcm::new(key);
        // 96-bits; unique per message
        let nonce = GenericArray::from_slice(b"unique nonce");

        let ciphertext = cipher.encrypt(nonce, b"plaintext message".as_ref())
            .expect("encryption failure!"); // NOTE: handle this error to avoid panics!

        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .expect("decryption failure!"); // NOTE: handle this error to avoid panics!
    }

    /// 解密
    pub fn decrypt(&self) {}
}


pub struct AeadAes256Gcm<'a> {
    plain_key: &'a [u8]
}

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

impl<'a> AeadAes256Gcm<'a> {
    fn creat(plain_key: &[u8]) -> AeadAes256Gcm { AeadAes256Gcm { plain_key } }
}

const AES_256_SZIE: usize = 32;

impl<'a> SsAead<[u8; AES_256_SZIE]> for AeadAes256Gcm<'a> {
    ///
    fn generate_sub_key(&self, salt_arr: &[u8]) -> Result<[u8; AES_256_SZIE]> {
        if salt_arr.len() != AES_256_SZIE { return Err(InvalidSaltSize(AES_256_SZIE as u8)); }
        let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), self.plain_key);
        let mut subkey = [0u8; AES_256_SZIE];
        hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
        return Ok(subkey);
    }
}


#[cfg(test)]
mod tests {
    use crate::encrypt::aead_aes_gcm::AeadAes256Gcm;
    use crate::encrypt::ss_aead::{AeadError, SsAead};

    #[test]
    fn it_works() {
        let aes256gcm = AeadAes256Gcm::creat(b"test");
        let salt = b"11111111111111111111111111111111";
        let result = aes256gcm.generate_sub_key(salt);
        match result {
            Ok(arr) => {
                println!("subkey(base64):{}", base64::encode(arr));
            }
            Err(err) => { assert!(false); }
        }
    }
}