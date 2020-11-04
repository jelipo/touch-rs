use std::pin::Pin;

use async_std::io::Read;
use async_std::net::{Ipv4Addr, TcpStream};
use async_std::stream::Stream;
use async_trait::async_trait;
use futures::AsyncReadExt;

#[async_trait(? Send)]
pub trait StreamAdapter {
    async fn get(&mut self);
}

pub struct SsStreamAdapter<'a> {
    stream: Box<&'a dyn Read>
}

impl<'a> SsStreamAdapter<'a> {
    pub fn new<R>(stream: &'a mut R) -> Self
        where R: Read + Unpin + Sized {
        SsStreamAdapter { stream: Box::new(stream) }
    }
}

#[async_trait(? Send)]
impl StreamAdapter for SsStreamAdapter<'_> {
    async fn get(&mut self) {
        let mut buf = [0u8; 32];
        let future = (*(self.stream)).read(&mut buf).await.unwrap();
    }
}

