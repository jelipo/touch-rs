use std::pin::Pin;

use async_std::io;
use async_std::io::prelude::Read;
use async_std::task::{Context, Poll};

mod stream;
