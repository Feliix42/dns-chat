use crate::dns::messages::DNSMessage;
use crate::dns::types::RecordData;
use crate::transport::ChatMessage;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener};
use std::sync::mpsc::{Receiver, RecvError};

pub fn run_sender(
    message_receiver: Receiver<ChatMessage>,
    listening_port: u16,
) -> Result<(), RecvError> {
    let mut buffer: VecDeque<RecordData> = VecDeque::new();
    // buffer for parsing incoming messages
    let mut reading_buffer = [0; 65535];

    let listener = TcpListener::bind(("0.0.0.0", listening_port))
        .expect("Could not bind listener to port. Is something else running?");
    listener
        .set_nonblocking(true)
        .expect("Could not move listener to non-blocking mode.");

    'inner: loop {
        // buffer as many messages as possible
        while let Ok(msg) = message_receiver.try_recv() {
            buffer.push_back(msg.into());
        }

        // see if a message request arrived
        match listener.accept() {
            Ok((mut socket, _remote_addr)) => {
                // answer the request if any data is available
                socket.set_nonblocking(false).unwrap();

                // read the request from the socket into a buffer & parse it
                let read_length = match socket.read(&mut reading_buffer) {
                    Ok(s) => s,
                    Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => continue 'inner,
                    Err(ref e) => panic!("{:?}", e),
                };

                if !buffer.is_empty() {
                    let packet_len =
                        u16::from_be_bytes([reading_buffer[0], reading_buffer[1]]) as usize;

                    if read_length != packet_len + 2 {
                        eprintln!(
                            "[sender] Received {} bytes, but packet reports {} bytes!",
                            read_length,
                            packet_len + 2
                        );
                    }

                    // NOTE(feliix42): RFC 1035, 4.2.2 - TCP usage requires prepending the message with 2
                    // bytes length information that does not include said two bytes
                    let parsed = DNSMessage::from(&reading_buffer[2..packet_len + 2]);

                    for msg in buffer.drain(..) {
                        // translate each message in a DNS reply & send it:
                        // - clone the received message, add reply
                        let mut reply = parsed.clone();
                        reply.add_answer(msg);
                        let mut sendable: Vec<u8> = reply.into();

                        // prepend the length of the message for TCP transfer
                        let len = u16::try_from(sendable.len()).unwrap();
                        let split = len.to_be_bytes();
                        sendable.insert(0, split[1]);
                        sendable.insert(0, split[0]);

                        // TODO(feliix42): Error handling
                        // - then send
                        socket.write(&sendable).unwrap();
                        socket.flush().unwrap();
                    }
                }

                // clear the buffer
                for i in 0..read_length {
                    // please rustc, vectorize this
                    reading_buffer[i] = 0;
                }
                socket.shutdown(Shutdown::Both).unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }
}
