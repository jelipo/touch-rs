// Or `Aes128Gcm`


use ring::aead;
use ring::aead::{Aad, BoundKey, Nonce};
use ring::error::Unspecified;

fn new_aes() -> std::io::Result<Vec<u8>> {
    let key = [0u8; 32];
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &key).unwrap();
    let nonce = AeadNonce::new();
    let mut sealing_key = aead::SealingKey::new(unbound_key, nonce);
    let mut vec = vec![0u8; 16];
    sealing_key.seal_in_place_append_tag(Aad::empty(), &mut vec);

    let _x = sealing_key.seal_in_place_separate_tag(Aad::empty(), vec.as_mut())
        .map(|tag| vec.extend(tag.as_ref()));

    Ok(vec)
}


pub struct AeadNonce {
    base_arr: [u8; 12],
}

impl aead::NonceSequence for AeadNonce {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        aead::Nonce::try_assume_unique_for_key(self.get_and_increment())
    }
}

impl AeadNonce {
    /// Get the nonce byte and increment
    pub fn get_and_increment(&mut self) -> &[u8] {
        self.increment(0);
        &self.base_arr
    }

    fn increment(&mut self, index: usize) {
        match self.base_arr[index] {
            255u8 => {
                self.increment(index + 1);
                self.base_arr[index] = 0;
            }
            n => self.base_arr[index] = n + 1,
        }
    }

    pub fn new() -> Self { Self { base_arr: [0u8; 12] } }
}


#[cfg(test)]
mod tests {
    use crate::encrypt::ss::a::new_aes;

    const NONCE: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    #[test]
    fn new_aes_test() {
        let vec = new_aes().unwrap();
        println!("{:?}", vec);
    }
}