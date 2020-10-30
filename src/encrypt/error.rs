use std::{fmt, result};

use crate::encrypt::error::EncryptError::InvalidSaltSize;

pub type Result<T> = result::Result<T, EncryptError>;

/// Errors about encrypt
pub enum EncryptError {
    InvalidLength(u8),
    // Invalid Slat size. u8: The right size
    InvalidSaltSize(u8),
    EncryptErr,
    DecryptErr,
}

impl fmt::Display for EncryptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt_err(self, f) }
}

impl fmt::Debug for EncryptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { fmt_err(self, f) }
}

fn fmt_err(aead_err: &EncryptError, f: &mut fmt::Formatter) -> fmt::Result {
    match aead_err {
        InvalidSaltSize(size) =>
            write!(f, "Invalid salt size.Right size:{}", size),
        n => write!(f, "Aead encrypt error: {}", n),
    }
}