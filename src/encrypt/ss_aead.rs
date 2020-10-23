use std::fmt::{Debug, Display, Formatter};
use std::{fmt, result};

use crate::encrypt::ss_aead::AeadError::InvalidSaltSize;

pub trait SsAead<K> {
    fn generate_sub_key(&self, salt_arr: &[u8]) -> Result<K,AeadError>;
}


///----------------------------------AEAD_ERROR--------------------------------


pub enum AeadError {
    InvalidLength(u8),
    // Invalid Slat size. u8: The right size
    InvalidSaltSize(u8),
}

impl fmt::Display for AeadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt_err(self, f) }
}

impl fmt::Debug for AeadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt_err(self, f) }
}

fn fmt_err(aead_err: &AeadError, f: &mut fmt::Formatter) -> fmt::Result {
    match aead_err {
        InvalidSaltSize(size) =>
            write!(f, "Invalid salt size.Right size:{}", size),
        n => write!(f, "Aead encrypt error: {}", n),
    }
}