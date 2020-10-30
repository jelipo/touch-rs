use md5::{Digest, Md5};
use crate::encrypt::error::EncryptError::InvalidSaltSize;
use hkdf::Hkdf;
use sha1::Sha1;
use crate::encrypt::error::Result;

mod ss_aead;
mod nonce;


/// Generate Shadowsocks master key.
fn gen_master_key(password: &[u8]) -> [u8; 32] {
    let md5_1_output = Md5::digest(password);
    let md5_1 = md5_1_output.as_slice();
    let mut md5_2_plain_vec = Vec::new();
    md5_2_plain_vec.extend_from_slice(md5_1);
    md5_2_plain_vec.append(&mut password.to_vec());
    let md5_2 = Md5::digest(&md5_2_plain_vec);
    let mut master_key = [0u8; 32];
    master_key[0..16].copy_from_slice(md5_1);
    master_key[16..32].copy_from_slice(md5_2.as_slice());
    return master_key;
}

const SS_SUBKEY_STR: &[u8] = b"ss-subkey";

fn generate_32_sub_key(salt_arr: &[u8], master_key: &[u8]) -> Result<[u8; 32]> {
    if salt_arr.len() != 32 { return Err(InvalidSaltSize(32 as u8)); }
    let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
    let mut subkey = [0u8; 32];
    hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
    return Ok(subkey);
}

fn generate_16_sub_key(salt_arr: &[u8], master_key: &[u8]) -> Result<[u8; 16]> {
    if salt_arr.len() != 16 { return Err(InvalidSaltSize(16 as u8)); }
    let hkdf = Hkdf::<Sha1>::new(Some(salt_arr), master_key);
    let mut subkey = [0u8; 16];
    hkdf.expand(SS_SUBKEY_STR, &mut subkey).unwrap();
    return Ok(subkey);
}
