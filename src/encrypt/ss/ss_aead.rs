use std::{fmt, result};
use std::fmt::{Debug, Display, Formatter};

use crate::encrypt::error::EncryptError;
use crate::encrypt::ss::ss_aead::EncryptError::InvalidSaltSize;

pub trait SsAead<K> {
    fn generate_sub_key(salt_arr: &[u8], master_key: &[u8]) -> Result<K, EncryptError>;
}


//----------------------------------AEAD_ERROR--------------------------------


