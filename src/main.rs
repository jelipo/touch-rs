use std::path::Path;

use async_std::io;
use async_std::net::{Shutdown, TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use async_std::task::JoinHandle;

use crate::core::config::ConfigReader;
use crate::core::selector::ProtocalSelector;
use crate::encrypt::aead::AeadType;
use crate::net::proxy::ProxyReader;
use crate::net::ss_stream::SsStreamReader;
use crate::socks::socks5::Socks5;

mod socks;
mod ss;
mod encrypt;
mod net;
mod core;
mod util;

#[async_std::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    //listen().await

    let path = Path::new("./conf/config.json");
    let reader = ConfigReader::read_config(path)?;
    ProtocalSelector::select(&reader).await
}

// async fn listen() -> io::Result<()> {
//     let listener = TcpListener::bind("127.0.0.1:3391").await.unwrap();
//     let mut incoming = listener.incoming();
//     let option: Option<io::Result<TcpStream>> = incoming.next().await;
//     let stream = option.unwrap().unwrap();
//
//     let mut reader = SsStreamReader::new(stream, "test", AeadType::AES256GCM);
//     let vec = reader.read().await?;
//     println!("Read:{:?}", vec);
//     let de_data = vec.as_slice();
//     let addrs = Socks5::read_to_socket_addrs(de_data);
//     println!("addr {:?}", addrs);
//     let data = &de_data[(addrs.1)..de_data.len()];
//     println!("{:?}", String::from_utf8(data.to_vec()).unwrap().as_str());
//     Ok(())
// }