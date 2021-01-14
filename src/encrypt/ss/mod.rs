use hkdf::Hkdf;
use md5::{Digest, Md5};
use sha1::Sha1;

use crate::encrypt::error::EncryptError::InvalidSaltSize;
use crate::encrypt::error::Result;
use ring::pbkdf2::PBKDF2_HMAC_SHA1;

pub mod ss_aead;

pub const DIGEST_LEN: usize = 16;

/// Generate master_key
/// Key derivation of OpenSSL's [EVP_BytesToKey](https://wiki.openssl.org/index.php/Manual:EVP_BytesToKey(3))
pub fn openssl_bytes_to_key(password: &[u8], key: &mut [u8]) {
    let key_len = key.len();

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
        key[offset..offset + amt].copy_from_slice(&digest[..amt]);

        offset += DIGEST_LEN;
        last_digest = Some(digest.into());
    }
}

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

fn generate_32_sub_key(salt_arr: &[u8], master_key: &[u8]) -> Result<[u8; 32]> {
    if salt_arr.len() != 32 { return Err(InvalidSaltSize(32)); }
    let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
    let mut subkey = [0u8; 32];
    hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
    Ok(subkey)
}

fn generate_16_sub_key(salt_arr: &[u8], master_key: &[u8]) -> Result<[u8; 16]> {
    if salt_arr.len() != 16 { return Err(InvalidSaltSize(16)); }
    let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
    let mut subkey = [0u8; 16];
    hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
    Ok(subkey)
}

fn generate_subkey(salt_arr: &[u8], master_key: &[u8], len: usize) -> Result<Box<[u8]>> {
    if salt_arr.len() != len { return Err(InvalidSaltSize(len)); }
    let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
    let mut subkey = vec![0u8; len];
    hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
    Ok(subkey.into_boxed_slice())
}