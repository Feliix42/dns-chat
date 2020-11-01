use std::net::TcpStream;
use std::io::{Read, Write};
use std::convert::TryFrom;

mod dns_messages;
mod types;

use dns_messages::DNSMessage;

fn main() -> std::io::Result<()> {
    let message = DNSMessage::new_request(23481, "ifsr.de".into());

    let mut stream = TcpStream::connect("192.168.178.44:53")?;

    let mut msg: Vec<u8> = message.into();

    // prepend the length of the message for TCP transfer
    let len = u16::try_from(msg.len()).unwrap();
    let split = len.to_be_bytes();
    msg.insert(0, split[1]);
    msg.insert(0, split[0]);

    stream.write(&msg)?;

    loop {
        let mut buf = [0; 500];
        stream.read(&mut buf)?;

        println!("Reply: {:?}", buf);
    }
}
