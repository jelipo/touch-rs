use md5::{Digest, Md5};

mod aead_aes_gcm;

mod ss_aead;


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