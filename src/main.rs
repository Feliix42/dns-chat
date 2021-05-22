use clap::Clap;
use dns::messages::DNSMessage;
use dns::types::RecordData;
use opts::Opts;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver, RecvError, SendError, Sender};
use std::thread;
use std::time::Duration;
use transport::ChatMessage;

mod dns;
mod opts;
mod state;
mod transport;
mod tui;

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();
    let Opts {
        target,
        target_port,
        listening_port,
    } = opts;

    let (msg_sender, rx) = mpsc::channel();
    let sender = thread::Builder::new().name("Sender".to_string());
    sender
        .spawn(move || run_sender(rx, listening_port))
        .expect("Could not spawn sender thread");

    let (sx, msg_recv) = mpsc::channel();
    let receiver = thread::Builder::new().name("Receiver".to_string());
    receiver
        .spawn(move || poll_messages(sx, (target, target_port)))
        .expect("Could not spawn receiver thread");

    if let Err(e) = tui::run(msg_sender, msg_recv) {
        eprintln!("{}", e);
    }

    Ok(())
}

fn run_sender(
    message_receiver: Receiver<ChatMessage>,
    listening_port: u16,
) -> Result<(), RecvError> {
    let mut buffer: VecDeque<RecordData> = VecDeque::new();
    // buffer for parsing incoming messages
    let mut reading_buffer = Vec::with_capacity(65535);

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
                if !buffer.is_empty() {
                    // read the request from the socket into a buffer & parse it
                    match socket.read_to_end(&mut reading_buffer) {
                        Ok(_) => (),
                        Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => continue 'inner,
                        Err(ref e) => panic!("{}", e),
                    }

                    let packet_len =
                        u16::from_be_bytes([reading_buffer[0], reading_buffer[1]]) as usize;

                    // NOTE(feliix42): RFC 1035, 4.2.2 - TCP usage requires prepending the message with 2
                    // bytes length information that does not include said two bytes
                    let parsed = DNSMessage::from(&reading_buffer[2..packet_len + 2]);
                    reading_buffer.clear();

                    for msg in buffer.drain(..) {
                        // translate each message in a DNS reply & send it:
                        // - clone the received message, increment the counter in the DNS message &
                        // add reply
                        // - then send
                        let mut reply = parsed.clone();
                        reply.add_answer(msg);
                        let sendable: Vec<u8> = reply.into();
                        // TODO(feliix42): Error handling
                        socket.write_all(&sendable).unwrap();
                    }
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }
}

fn poll_messages<A: ToSocketAddrs + Clone>(
    message_sender: Sender<ChatMessage>,
    target: A,
) -> Result<(), SendError<ChatMessage>> {
    // poll every x seconds for new messages (10s?)
    // Read until connection is reset (catch that error!)
    // sleep
    let mut received: VecDeque<DNSMessage> = VecDeque::new();
    let mut buf = Vec::with_capacity(65535);

    loop {
        let message = DNSMessage::new_request(23481, "ifsr.de".into());
        let mut msg: Vec<u8> = message.into();

        // prepend the length of the message for TCP transfer
        let len = u16::try_from(msg.len()).unwrap();
        let split = len.to_be_bytes();
        msg.insert(0, split[1]);
        msg.insert(0, split[0]);

        let mut stream =
            TcpStream::connect(target.clone()).expect("Couldn't connect to target DNS server");
        // let mut stream = TcpStream::connect("192.168.178.44:53")?;

        // TODO(feliix42): error handling
        stream.write_all(&msg).unwrap();

        // receive messages until everything has been transmitted
        'inner: loop {
            // TODO(feliix42): size ok? -> optimizations for less allocations?
            // TODO(feliix42): does this read beyond the borders of individual packets?
            match stream.read_to_end(&mut buf) {
                Ok(_) => (),
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => break 'inner,
                Err(ref e) => panic!("{}", e),
            }

            let packet_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;

            // NOTE(feliix42): RFC 1035, 4.2.2 - TCP usage requires prepending the message with 2
            // bytes length information that does not include said two bytes
            let parsed = DNSMessage::from(&buf[2..packet_len + 2]);
            buf.clear();
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

        thread::sleep(Duration::from_secs(5));
    }
}
