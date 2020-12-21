// Or `Aes128Gcm`
use aes_gcm::aead::{AeadInPlace, generic_array::GenericArray, NewAead};
use aes_gcm::Aes256Gcm;

fn new_aes() {
    let key = GenericArray::from_slice(b"an example very very secret key.");
    let cipher = Aes256Gcm::new(key);

    let nonce = GenericArray::from_slice(b"unique nonce"); // 96-bits; unique per message

    let mut buffer = vec![0u8; 128];
    let x = b"plaintext message";
    buffer[..17].copy_from_slice(x);
    println!("{:?}", buffer);


    // Encrypt `buffer` in-place, replacing the plaintext contents with ciphertext
    cipher.encrypt_in_place(nonce, b"", &mut buffer).expect("encryption failure!");

    println!("{:?}", buffer);

    // `buffer` now contains the message ciphertext
    assert_ne!(&buffer, b"plaintext message");

    // Decrypt `buffer` in-place, replacing its ciphertext context with the original plaintext
    cipher.decrypt_in_place(nonce, b"", &mut buffer).expect("decryption failure!");
    assert_eq!(&buffer, b"plaintext message");
}

#[cfg(test)]
mod tests {
    use crate::encrypt::ss::a::new_aes;

    const NONCE: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    #[test]
    fn new_aes_test() {
        new_aes();
    }
}