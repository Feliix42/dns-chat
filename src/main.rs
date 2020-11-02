use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::TcpStream;

mod dns_messages;
mod types;

use dns_messages::DNSMessage;

fn main() -> std::io::Result<()> {
    let message = DNSMessage::new_request(23481, "ifsr.de".into());

    let mut stream = TcpStream::connect("192.168.178.44:53")?;

    let mut msg: Vec<u8> = message.into();
    println!("{:?}", msg);

    // prepend the length of the message for TCP transfer
    let len = u16::try_from(msg.len()).unwrap();
    let split = len.to_be_bytes();
    msg.insert(0, split[1]);
    msg.insert(0, split[0]);

    stream.write(&msg)?;

    loop {
        let mut buf = [0; 500];
        let len = stream.read(&mut buf)?;

        println!("Reply: {:?}", buf);
        let parsed = DNSMessage::from(&buf[2..len + 2]);
        println!("{:#?}", parsed);
    }
}
