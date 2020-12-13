use std::io;

use async_std::net::TcpStream;

use crate::net::AddressType;
use crate::net::proxy::{OutProxyStarter, OutputProxy, ProxyInfo, ProxyReader, ProxyWriter, Closer};
use crate::util::address::Address;

//------------------------------SS_OUT_PROXY-----------------------------------------
#[derive(Clone)]
pub struct PlainOutProxy {}

impl PlainOutProxy {
    pub fn new() -> Self {
        Self {}
    }
}

impl OutputProxy for PlainOutProxy {
    fn gen_starter(&mut self) -> io::Result<Box<dyn OutProxyStarter>> {
        Ok(Box::new(PlainOutProxyStarter {}))
    }
}

pub struct PlainOutProxyStarter {}

#[async_trait]
impl OutProxyStarter for PlainOutProxyStarter {
    async fn new_connect(&mut self, info: ProxyInfo) ->
    io::Result<(Box<dyn ProxyReader>, Box<dyn ProxyWriter>, Box<dyn Closer>)> {
        debug!("new plain out connect");
        let tcpstream = Address::new_connect(&info.address, info.port, &info.address_type)?;


        Ok((Box::new(reader), Box::new(writer), Box::new(closer)))
    }
}
//<--<--<--<--<--<--<--<--<--<--<--<--SS_OUT_PROXY--<--<--<--<--<--<--<--<--<--<--<--<
