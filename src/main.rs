use std::io;
use std::path::Path;

use bytes::BufMut;

use crate::core::config::ConfigReader;
use crate::core::selector::ProtocalSelector;

mod socks;
mod ss;
mod encrypt;
mod net;
mod core;
mod util;

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    // test_bytes();
    // Ok(())

    //listen().await

    let path = Path::new("./conf/config.json");
    let reader = ConfigReader::read_config(path)?;
    ProtocalSelector::select(&reader).await
}

// async fn listen() -> io::Result<()> {
//     let listener = TcpListener::bind("127.0.0.1:3391").await.unwrap();
//     let (stream, addr) = listener.accept().await?;
//     let mut reader = SsStreamReader::new(stream, "test", AeadType::AES128GCM);
//     let de_data = reader.read().await?;
//     println!("Read:{:?}", de_data);
//     let addrs = Socks5::read_to_socket_addrs(de_data);
//     println!("addr {:?}", addrs);
//     let data = &de_data[(addrs.1)..de_data.len()];
//     println!("{:?}", String::from_utf8(data.to_vec()).unwrap().as_str());
//     Ok(())
// }

fn test_bytes() {
    let mut arr = vec![];
    arr.put(&b"aaaa"[..]);
    println!("{:?}", arr);

    (&mut arr[..1]).put(&b"bb"[..]);
    println!("{:?}", arr);
}