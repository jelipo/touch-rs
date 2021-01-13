use std::io;


use async_std::io::ReadExt;
use async_std::net::{TcpStream};
use async_std::prelude::*;


use crate::net::proxy::{Closer, OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter};


#[derive(Clone)]
pub struct PlainOutProxy {}

impl PlainOutProxy {
    pub fn new() -> Self {
        Self {}
    }
}

// impl OutputProxy for PlainOutProxy {
//     fn gen_starter(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
//         Ok(Box::new(PlainOutProxyStarter {}))
//     }
// }

// pub struct PlainOutProxyStarter {}
//
// #[async_trait]
// impl OutProxyStarter for PlainOutProxyStarter {
//     async fn new_connect(&mut self, info: ProxyInfo) ->
//     io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>, Box<dyn Closer>)> {
//         debug!("new plain out connect");
//         let tcpstream = Address::new_connect(&info.address, info.port, &info.address_type)?;
//
//
//         Ok((Box::new(reader), Box::new(writer), Box::new(closer)))
//     }
// }


struct PlainReader {
    tcpstream: TcpStream,
    read_buf: Box<[u8]>,
}

impl PlainReader {
    async fn read_data(&mut self) -> io::Result<&[u8]> {
        let size = self.tcpstream.read(self.read_buf.as_mut()).await?;
        Ok(&self.read_buf[..size])
    }
}

// #[async_trait]
// impl ProxyReader for PlainReader {
//     async fn read(&mut self) -> io::Result<Vec<u8>> {
//         let size = self.tcpstream.read(self.read_buf.as_mut()).await?;
//         let vec = self.read_buf[..size].to_vec();
//     }
// }
