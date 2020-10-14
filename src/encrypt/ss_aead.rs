pub trait SsAead<K> {
    fn generate_sub_key(plain_key: &str, salt: &str) -> K;
}