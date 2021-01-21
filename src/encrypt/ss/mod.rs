use hkdf::Hkdf;
use md5::{Digest, Md5};
use sha1::Sha1;


use crate::encrypt::error::Result;


pub mod ss_aead;

pub const DIGEST_LEN: usize = 16;

/// Generate master_key
/// Key derivation of OpenSSL's [EVP_BytesToKey](https://wiki.openssl.org/index.php/Manual:EVP_BytesToKey(3))
pub fn openssl_bytes_to_key(password: &[u8], key_buffer: &mut [u8]) {
    let key_len = key_buffer.len();
    let mut last_digest: Option<[u8; DIGEST_LEN]> = None;
    let mut offset = 0usize;
    while offset < key_len {
        let mut m = Md5::new();
        if let Some(digest) = last_digest {
            m.update(&digest);
        }
        m.update(password);
        let digest = m.finalize();
        let amt = std::cmp::min(key_len - offset, DIGEST_LEN);
        key_buffer[offset..offset + amt].copy_from_slice(&digest[..amt]);
        offset += DIGEST_LEN;
        last_digest = Some(digest.into());
    }
}

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

fn generate_subkey(salt_arr: &[u8], master_key: &[u8]) -> Result<Box<[u8]>> {
    let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
    let mut subkey = vec![0u8; salt_arr.len()];
    hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
    Ok(subkey.into_boxed_slice())
}