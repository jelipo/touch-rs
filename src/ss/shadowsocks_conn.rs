use aes::Aes256;
use cfb_mode::Cfb;

pub struct ShadowsocksConnector {}

type Aes256Cfb = Cfb<Aes256>;


impl ShadowsocksConnector {
    pub fn test(&self) {
        let key = "";
        let iv = "";

    }
}

