use aes::Aes256;
use cfb_mode::Cfb;
use cfb_mode::stream_cipher::NewStreamCipher;

type Aes256CfbEncrypt = Cfb<Aes256>;

pub struct Aes256Cfb {
    encrypt: Cfb<Aes256>,
}

impl Aes256Cfb {
    /// Creat a new AES-256-CFB type
    pub fn new(key: &[u8; 32], iv: &[u8; 16]) -> Self {
        Aes256Cfb {
            encrypt: Aes256CfbEncrypt::new_var(key, iv).unwrap()
        }
    }


    /// 加密
    pub fn encrypt(&self) {
        let mut data = b"hello".to_vec();
        let key = b"smkldospdosldaaasmkldospdosldaaa";
        let iv = b"0392039203920300";
        println!("{:?}", data);
        let cfb = Aes256CfbEncrypt::new_var(key, iv).unwrap();
        println!("{:?}", data);
    }

    /// 解密
    pub fn decrypt(&self) {

    }
}
