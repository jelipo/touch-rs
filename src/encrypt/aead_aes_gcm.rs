use aes_gcm::{Aes256Gcm, Error};
use aes_gcm::aead::{Aead, generic_array::GenericArray, NewAead};
use aes_gcm::aes::block_cipher::BlockCipherMut;
use hkdf::Hkdf;
use ring::aead::{Aad, AES_256_GCM, BoundKey, Nonce, NonceSequence, OpeningKey, UnboundKey};
use ring::error::Unspecified;
use sha1::Sha1;

use crate::encrypt::ss_aead::{AeadError, SsAead};
use crate::encrypt::ss_aead::AeadError::InvalidSaltSize;

pub struct AeadAes256Gcm<'a> {
    plain_key: &'a [u8]
}

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

impl<'a> AeadAes256Gcm<'a> {
    pub fn creat(plain_key: &[u8]) -> AeadAes256Gcm { AeadAes256Gcm { plain_key } }

    /// 加密
    pub fn encrypt(
        &self, subkey: &[u8], nonce_arr: &[u8], data: &[u8],
    ) -> Vec<u8> {
        let key = GenericArray::from_slice(subkey);
        let cipher = Aes256Gcm::new(key);
        let nonce = GenericArray::from_slice(nonce_arr);
        return cipher.encrypt(nonce, data)
            .expect("encryption failure!");
    }

    /// 解密
    pub fn decrypt(
        &self, subkey: &[u8], nonce_arr: &[u8], data: &[u8],
    ) -> std::result::Result<Vec<u8>, Error> {
        let key = GenericArray::from_slice(subkey);
        let cipher = Aes256Gcm::new(key);
        let nonce = GenericArray::from_slice(nonce_arr);
        cipher.decrypt(nonce, data)
    }
}

const AES_256_SZIE: usize = 32;

impl<'a> SsAead<[u8; AES_256_SZIE]> for AeadAes256Gcm<'a> {
    ///
    fn generate_sub_key(&self, salt_arr: &[u8]) -> Result<[u8; AES_256_SZIE], AeadError> {
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
        let salt = &[211, 243, 92, 14, 14, 171, 116, 2, 18, 144, 85, 251, 115, 120, 138, 107, 76, 128, 70, 116, 16, 212, 237, 9, 33, 254, 32, 220, 114, 13, 229, 23];
        println!("salt {:?}", hex::encode(salt));

        let aes256gcm = AeadAes256Gcm::creat(b"laocao");
        let subkey = aes256gcm.generate_sub_key(salt).unwrap();
        println!("subkey {:?}", base64::encode(subkey));

        let nonce: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let plain_text = b"hello";
        let en_vec = aes256gcm.encrypt(&subkey, &nonce, plain_text);
        println!("en size {} arr:{:?}", en_vec.len(), en_vec);

        let de_vec = aes256gcm.decrypt(&subkey, &nonce, en_vec.as_slice()).unwrap();
        println!("de size {} arr:{:?}", de_vec.len(), de_vec);
        println!("{:?}", String::from_utf8(de_vec).unwrap());
    }

    #[test]
    fn custom() {
        let aes256gcm = AeadAes256Gcm::creat(b"laocao");
        let salt: &[u8; 32] = &[86, 67, 86, 47, 152, 144, 239, 240, 86, 40, 201, 92, 59, 175, 206,
            67, 110, 70, 44, 215, 175, 185, 205, 118, 181, 99, 233, 169, 118, 182, 86, 130];
        let subkey = aes256gcm.generate_sub_key(salt).unwrap();
        println!("subkey {}", base64::encode(subkey));
        let en_vec: [u8; 18] = [114, 73, 229, 154, 157, 253, 172, 173, 223, 46, 92, 133, 153, 187, 121, 223, 31, 30];
        println!("en {}", hex::encode(en_vec));
        //let subkey1 = base64::decode("4bWZVS0Wck2r588czdxR7qBH4W8=").unwrap().as_slice();
        let nonce_arr: &[u8; 12] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        println!("{}", base64::encode(nonce_arr));
        let de_vec = aes256gcm.decrypt(&subkey, nonce_arr, &en_vec);
        match de_vec {
            Ok(result) => {
                println!("{:?}", result);
                return;
            }
            Err(e) => { println!("解密失败 {}", e); }
        }
    }
}