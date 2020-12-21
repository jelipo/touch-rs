use hkdf::Hkdf;
use md5::{Digest, Md5};
use sha1::Sha1;

use crate::encrypt::error::EncryptError::InvalidSaltSize;
use crate::encrypt::error::Result;

pub mod ss_aead;
mod a;


/// Generate Shadowsocks master key.
fn gen_master_key(password: &[u8]) -> [u8; 32] {
    let md5_1_output = Md5::digest(password);
    let md5_1 = md5_1_output.as_slice();
    let mut md5_2_plain_vec = vec![0u8; 16 + password.len()];
    md5_2_plain_vec[..16].copy_from_slice(md5_1);
    md5_2_plain_vec[16..password.len() + 16].copy_from_slice(password);
    let md5_2 = Md5::digest(&md5_2_plain_vec);
    let mut master_key = [0u8; 32];
    master_key[..16].copy_from_slice(md5_1);
    master_key[16..32].copy_from_slice(&md5_2);
    master_key
}

const STR: &[u8] = b"test";


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