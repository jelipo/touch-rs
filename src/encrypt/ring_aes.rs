use hkdf::Hkdf;
use ring::aead::{Aad, AES_256_GCM, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey};
use ring::error::Unspecified;
use sha1::Sha1;

use crate::encrypt::ss_aead::AeadError;
use crate::encrypt::ss_aead::AeadError::InvalidSaltSize;

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

pub struct RingAeadNonceSequence {
    nonce: [u8; 12],
}

impl RingAeadNonceSequence {
    fn new(nonce: [u8; 12]) -> RingAeadNonceSequence {
        RingAeadNonceSequence {
            nonce
        }
    }
}

impl NonceSequence for RingAeadNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.nonce);
        Ok(nonce)
    }
}

struct RingEn {}

impl RingEn {
    /// 加密
    pub fn encrypt_ring(
        &self, subkey: &[u8], nonce_arr: [u8; 12], data: &mut Vec<u8>,
    ) {
        let key_type = UnboundKey::new(&AES_256_GCM, subkey).unwrap();
        let sequence = RingAeadNonceSequence::new(nonce_arr);
        let mut sealing_key = SealingKey::new(key_type, sequence);
        sealing_key.seal_in_place_append_tag(Aad::empty(), data).unwrap();
    }

    /// 加密
    pub fn decrypt_ring<'a>(
        &self, subkey: &[u8], nonce_arr: [u8; 12], data: &'a mut [u8],
    ) -> &'a mut [u8] {
        let key_type = UnboundKey::new(&AES_256_GCM, subkey).unwrap();
        let sequence = RingAeadNonceSequence::new(nonce_arr);
        let mut opening_key = OpeningKey::new(key_type, sequence);
        let x = opening_key.open_in_place(Aad::empty(), data).unwrap();
        return x;
    }

    ///
    fn generate_sub_key(&self, salt_arr: &[u8], key: &[u8]) -> Result<[u8; 32], AeadError> {
        let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), key);
        let mut subkey = [0u8; 32];
        hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
        return Ok(subkey);
    }

    pub fn new() -> Self {
        RingEn {}
    }
}


#[cfg(test)]
mod tests {
    use ring::aead::Aad;

    use crate::encrypt::aead_aes_gcm::AeadAes256Gcm;
    use crate::encrypt::ring_aes::RingEn;
    use crate::encrypt::ss_aead::{AeadError, SsAead};

    #[test]
    fn full() {
        let aes256gcm = RingEn::new();
        let salt: &[u8; 32] = &[153, 71, 45, 178, 62, 121, 54, 201, 254, 253, 99, 80, 146, 109, 34, 60, 199, 21, 220, 126, 223, 88, 157, 171, 165, 158, 87, 7, 240, 29, 48, 115];
        let subkey = aes256gcm.generate_sub_key(salt, b"laocao").unwrap();

        let mut nonce_arr = [0u8; 12];
        let mut data_arr: [u8; 18] = [240, 217, 236, 53, 227, 253, 8, 89, 158, 112, 11, 246, 109, 28, 30, 86, 92, 19];
        let mut data = Vec::from(data_arr);

        //
        // let en_vec = aes256gcm.encrypt_ring(subkey, nonce_arr, &mut data);
        // println!("{}", hex::encode(&data));


        let x = aes256gcm.decrypt_ring(&subkey, nonce_arr, &mut data_arr);
        println!("{:?}", hex::encode(x));
    }
}