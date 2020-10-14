use aes_gcm::aead::{Aead, generic_array::GenericArray, NewAead};
use aes_gcm::Aes256Gcm;
use hkdf::Hkdf;
use sha1::Sha1;

use crate::encrypt::ss_aead::SsAead;

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


pub struct AeadAes256Gcm {
    plain_key: Box<[u8]>
}

impl AeadAes256Gcm {
    fn creat(plain_key: &str) -> AeadAes256Gcm {
        AeadAes256Gcm {
            plain_key: Box::new(().)
        }
    }
}

impl SsAead<[u8; 32]> for AeadAes256Gcm {
    fn generate_sub_key(plain_key: &str, salt: &str) -> [u8; 32] {
        let salt_arr = salt.as_bytes();
        let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), plain_key.as_bytes());
        let mut subkey = [0u8; 32];
        hkdf.expand(b"ss-subkey", &mut subkey).unwrap();
        return subkey;
    }
}


