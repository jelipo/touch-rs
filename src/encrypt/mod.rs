use ring::error::Unspecified;

pub mod aead;
pub mod error;
pub mod ss;

pub struct Nonce {
    base_arr: [u8; 12],
    first: bool,
}

impl Nonce {
    /// Get the nonce byte and increment
    pub fn get_and_increment(&mut self) -> &[u8] {
        if self.first {
            self.first = false;
            &self.base_arr
        } else {
            self.increment(0);
            &self.base_arr
        }
    }

    fn increment(&mut self, index: usize) {
        let x = self.base_arr[index];
        if x == 255u8 {
            self.increment(index + 1);
            self.base_arr[index] = 0;
        } else {
            self.base_arr[index] = x + 1;
        }
    }

    pub fn new() -> Self {
        Nonce {
            base_arr: [0u8; 12],
            first: true,
        }
    }
}

impl ring::aead::NonceSequence for Nonce {
    fn advance(&mut self) -> Result<ring::aead::Nonce, Unspecified> {
        ring::aead::Nonce::try_assume_unique_for_key(self.get_and_increment())
    }
}
