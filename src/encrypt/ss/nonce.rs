use std::ops::Add;

pub struct Nonce {
    base: u128
}

impl Nonce {
    fn add(&mut self) -> Box<[u8; 12]> {
        self.base = self.base + 1;
        let byt: [u8; 16] = self.base.to_be_bytes();
        let x: [u8] = byt[4..16];
        Box::new(x)
    }
}