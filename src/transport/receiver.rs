use crate::dns::messages::DNSMessage;
use crate::transport::ChatMessage;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{SendError, Sender};
use std::thread;
use std::time::Duration;

pub fn poll_messages<A: ToSocketAddrs + Clone>(
    message_sender: Sender<ChatMessage>,
    target: A,
) -> Result<(), SendError<ChatMessage>> {
    // poll every x seconds for new messages (10s?)
    // Read until connection is reset (catch that error!)
    // sleep
    let mut received: VecDeque<DNSMessage> = VecDeque::new();
    let mut buf = [0; 65535];

    loop {
        let message = DNSMessage::new_request(23481, "ifsr.de".into());
        let mut msg: Vec<u8> = message.into();

        // prepend the length of the message for TCP transfer
        let len = u16::try_from(msg.len()).unwrap();
        let split = len.to_be_bytes();
        msg.insert(0, split[1]);
        msg.insert(0, split[0]);

        let mut stream = match TcpStream::connect(target.clone()) {
            Ok(con) => con,
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                // Couldn't connect to target DNS server
                thread::sleep(Duration::from_secs(5));
                continue;
            }
            Err(e) => panic!("{}", e),
        };

        // TODO(feliix42): error handling
        stream.write(&msg).unwrap();
        stream.flush().unwrap();

        // receive messages until everything has been transmitted
        'inner: loop {
            // TODO(feliix42): does this read beyond the borders of individual packets?
            let read_length = match stream.read(&mut buf) {
                Ok(0) => {
                    //eprintln!("[receiver] received empty message");
                    break 'inner;
                }
                Ok(sz) => sz,
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => break 'inner,
                Err(ref e) => panic!("{}", e),
            };

            let packet_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;

            // NOTE(feliix42): RFC 1035, 4.2.2 - TCP usage requires prepending the message with 2
            // bytes length information that does not include said two bytes
            let parsed = DNSMessage::from(&buf[2..packet_len + 2]);

            // clear the buffer
            for i in 0..read_length {
                buf[i] = 0;
            }
            received.push_back(parsed);
        }

        if !received.is_empty() {
            // is messages were received, convert them and send them back to the main thread
            for msg in received.drain(..) {
                let chat_messages = ChatMessage::from_dns(msg);
                for chat_msg in chat_messages {
                    message_sender.send(chat_msg)?;
                }
            }
        }

        match stream.shutdown(Shutdown::Both) {
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::NotConnected => (),
            Err(e) => panic!("{}", e),
        }

        thread::sleep(Duration::from_secs(5));
    }
}
