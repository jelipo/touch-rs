use std::pin::Pin;

use async_std::io;
use async_std::io::prelude::Read;
use async_std::task::{Context, Poll};

mod stream;

pub struct SsStream {}

impl Read for SsStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {}
}