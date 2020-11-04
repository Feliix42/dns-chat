use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender, SendError, Receiver, RecvError};
use std::collections::VecDeque;
use std::thread;

mod dns;

use dns::messages::DNSMessage;

fn main() -> std::io::Result<()> {
    // let inp: Vec<u8> = vec![0, 148, 91, 185, 129, 128, 0, 1, 0, 2, 0, 0, 0, 0, 4, 105, 102, 115, 114, 2, 100, 101, 0, 0, 16, 0, 1, 192, 12, 0, 16, 0, 1, 0, 0, 2, 88, 0, 30, 29, 118, 61, 115, 112, 102, 49, 32, 105, 112, 52, 58, 49, 52, 49, 46, 51, 48, 46, 51, 48, 46, 49, 51, 48, 32, 126, 97, 108, 108, 192, 12, 0, 16, 0, 1, 0, 0, 2, 88, 0, 69, 68, 103, 111, 111, 103, 108, 101, 45, 115, 105, 116, 101, 45, 118, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 61, 55, 48, 45, 68, 85, 116, 65, 111, 49, 68, 103, 76, 119, 85, 120, 57, 74, 110, 106, 77, 110, 56, 77, 103, 95, 81, 57, 83, 95, 119, 115, 75, 51, 100, 52, 115, 50, 115, 113, 88, 69, 101, 56];

    // let parsed = dbg!(DNSMessage::from(inp.as_slice()));
    let (sx, rx) = mpsc::channel();
    thread::spawn(move || run_sender(rx));

    let message = DNSMessage::new_request(23481, "ifsr.de".into());

    let mut stream = TcpStream::connect("127.0.0.1:53")?;
    // let mut stream = TcpStream::connect("192.168.178.44:53")?;

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

fn run_sender(message_receiver: Receiver<String>) -> Result<(), RecvError> {
    let mut buffer = VecDeque::new();
    
    let listener = TcpListener::bind("0.0.0.0:53").expect("Could not bind listener to port 53. Is another DNS server running?");
    listener.set_nonblocking(true).expect("Could not move listener to non-blocking mode.");

    loop {
        // buffer as many messages as possible
        while let Ok(msg) = message_receiver.try_recv() {
            buffer.push_back(msg);
        }

        // see if a message request arrived
        match listener.accept() {
            Ok((mut socket, remote_addr)) => {
                // answer the request
                println!("Got packet from {}", remote_addr);
                let inp: Vec<u8> = vec![0, 148, 91, 185, 129, 128, 0, 1, 0, 2, 0, 0, 0, 0, 4, 105, 102, 115, 114, 2, 100, 101, 0, 0, 16, 0, 1, 192, 12, 0, 16, 0, 1, 0, 0, 2, 88, 0, 30, 29, 118, 61, 115, 112, 102, 49, 32, 105, 112, 52, 58, 49, 52, 49, 46, 51, 48, 46, 51, 48, 46, 49, 51, 48, 32, 126, 97, 108, 108, 192, 12, 0, 16, 0, 1, 0, 0, 2, 88, 0, 69, 68, 103, 111, 111, 103, 108, 101, 45, 115, 105, 116, 101, 45, 118, 101, 114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 61, 55, 48, 45, 68, 85, 116, 65, 111, 49, 68, 103, 76, 119, 85, 120, 57, 74, 110, 106, 77, 110, 56, 77, 103, 95, 81, 57, 83, 95, 119, 115, 75, 51, 100, 52, 115, 50, 115, 113, 88, 69, 101, 56];
                socket.write(&inp).unwrap();
                // TODO: Split the message in small parts, assign them to packets
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }
}

fn poll_messages(message_sender: Sender<String>) -> Result<(), SendError<String>> {
    // poll every x seconds for new messages (10s?)
    // Read until connection is reset (catch that error!)
    // sleep

    loop {
        
    }
}
