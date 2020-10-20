use aes_gcm::{Aes256Gcm, Error};
use aes_gcm::aead::{Aead, generic_array::GenericArray, NewAead};
use hkdf::Hkdf;
use sha1::Sha1;

use crate::encrypt::ss_aead::{AeadError, SsAead};
use crate::encrypt::ss_aead::AeadError::InvalidSaltSize;
use crate::encrypt::ss_aead::Result;

pub struct AeadAes256Gcm<'a> {
    plain_key: &'a [u8]
}

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

impl<'a> AeadAes256Gcm<'a> {
    fn creat(plain_key: &[u8]) -> AeadAes256Gcm { AeadAes256Gcm { plain_key } }

    /// 加密
    pub fn encrypt(
        &self, subkey: &[u8; AES_256_SZIE], nonce_arr: &[u8; 12], data: &[u8],
    ) -> Vec<u8> {
        let key = GenericArray::from_slice(subkey);
        let cipher = Aes256Gcm::new(key);
        let nonce = GenericArray::from_slice(nonce_arr);
        cipher.encrypt(nonce, data).unwrap()
    }

    /// 解密
    pub fn decrypt(
        &self, subkey: &[u8; AES_256_SZIE], nonce_arr: &[u8; 12], data: &[u8],
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

// #[test]
    // fn it_works() {
    //     let aes256gcm = AeadAes256Gcm::creat(b"laocao");
    //     let salt = b"11111111111111111111111111111111";
    //     let subkey = aes256gcm.generate_sub_key(salt).unwrap();
    //     let nonce: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //     let plain_text = "悲伤的故事".as_bytes();
    //     let en_vec = aes256gcm.encrypt(&subkey, &nonce, plain_text);
    //     println!("en size {} arr:{:?}", en_vec.len(), en_vec);
    //
    //     let de_vec = aes256gcm.decrypt(&subkey, &nonce, en_vec.as_slice());
    //     println!("de size {} arr:{:?}", de_vec.len(), de_vec);
    //     println!("{:?}", String::from_utf8(de_vec.unwrap()).unwrap());
    // }

    #[test]
    fn custom() {
        let aes256gcm = AeadAes256Gcm::creat(b"laocao");
        let salt: [u8; 32] = [197, 130, 234, 183, 188, 221, 101, 19, 196, 47, 180, 128, 105, 69, 109, 211, 183, 113, 184, 89, 70, 206, 159, 100, 20, 108, 2, 104, 36, 125, 170, 44];
        let subkey = aes256gcm.generate_sub_key(&salt).unwrap();
        let mut nonce: u128 = 0;
        //let plain_text = "11".as_bytes();
        //let en_vec = aes256gcm.encrypt(&subkey, &nonce, plain_text);
        // println!("en size {} arr:{:?}", en_vec.len(), en_vec);
        loop {
            nonce = nonce + 1;
            let nonce_full = nonce.to_be_bytes();
            let mut nonce_arr = [0u8; 12];
            for i in 0..12 {
                nonce_arr[i] = nonce_full[i + 4];
            }
            let en_vec: [u8; 18] = [213, 201, 37, 188, 204, 237, 156, 223, 224, 35, 203, 208, 179, 31, 91, 181, 20, 246];
            let de_vec = aes256gcm.decrypt(&subkey, &nonce_arr, &en_vec);
            match de_vec {
                Ok(result) => {
                    println!("{:?}", result);
                    return;
                }
                _ => ()
            }
        }

        //println!("de size {} arr:{:?}", de_vec.len(), de_vec);
        //println!("{:?}", String::from_utf8(de_vec.unwrap()).unwrap());
    }
}