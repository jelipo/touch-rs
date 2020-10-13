use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};
use aes_gcm::Aes256Gcm;
use hkdf::Hkdf;
use sha1::Sha1;


pub struct AesGcmAead {}

impl AesGcmAead {
    ///
    pub fn new(key: &[u8; 32], iv: &[u8; 16]) -> Self {
        AesGcmAead {}
    }


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
        Hkdf::<Sha1>::new(Some(&salt[..]), &ikm);
    }

    /// 解密
    pub fn decrypt(&self) {}
}
