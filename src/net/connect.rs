use std::net::TcpStream;
use std::io::Write;

trait Client {}

trait Server {

    fn test() {
        let mut stream = TcpStream::connect("").unwrap();
        let buf = [0u8; 128];
        let result = stream.write(&buf[0..64]).unwrap();
    }


}