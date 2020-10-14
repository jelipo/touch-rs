use hkdf::Hkdf;
use sha1::Sha1;

struct SsEncryptUtil {}

impl SsEncryptUtil {
    ///
    pub fn generate_sub_key(plain_key: &str, salt: &str) -> [u8; 48] {
        let salt_arr = salt.as_bytes();
        let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), plain_key.as_bytes());
        let mut subkey = [0u8; 48];
        hkdf.expand(b"ss-subkey", &mut subkey).unwrap();
        println!("{:?}", subkey);
        return subkey;
    }
}


#[cfg(test)]
mod tests {
    use crate::encrypt::ss_encrypt_util::SsEncryptUtil;

    #[test]
    fn it_works() {
        //let gcm = SsEncryptUtil::generate_sub_key();
        assert_eq!(2 + 2, 4);
    }
}