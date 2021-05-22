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

    let listener = TcpListener::bind(("0.0.0.0", listening_port))
        .expect("Could not bind listener to port. Is something else running?");
    listener
        .set_nonblocking(true)
        .expect("Could not move listener to non-blocking mode.");

    loop {
        // buffer as many messages as possible
        while let Ok(msg) = message_receiver.try_recv() {
            buffer.push_back(msg.into());
        }

        // see if a message request arrived
        match listener.accept() {
            Ok((mut socket, _remote_addr)) => {
                // println!("Got packet from {}", remote_addr);
                // answer the request if any data is available
                //if !buffer.is_empty() {
                //let mut msg = socket.
                //for msg in buffer.drain(..) {
                //let transferable = msg.
                //}
                //}
                let inp: Vec<u8> = vec![
                    0, 148, 91, 185, 129, 128, 0, 1, 0, 2, 0, 0, 0, 0, 4, 105, 102, 115, 114, 2,
                    100, 101, 0, 0, 16, 0, 1, 192, 12, 0, 16, 0, 1, 0, 0, 2, 88, 0, 30, 29, 118,
                    61, 115, 112, 102, 49, 32, 105, 112, 52, 58, 49, 52, 49, 46, 51, 48, 46, 51,
                    48, 46, 49, 51, 48, 32, 126, 97, 108, 108, 192, 12, 0, 16, 0, 1, 0, 0, 2, 88,
                    0, 69, 68, 103, 111, 111, 103, 108, 101, 45, 115, 105, 116, 101, 45, 118, 101,
                    114, 105, 102, 105, 99, 97, 116, 105, 111, 110, 61, 55, 48, 45, 68, 85, 116,
                    65, 111, 49, 68, 103, 76, 119, 85, 120, 57, 74, 110, 106, 77, 110, 56, 77, 103,
                    95, 81, 57, 83, 95, 119, 115, 75, 51, 100, 52, 115, 50, 115, 113, 88, 69, 101,
                    56,
                ];
                socket.write_all(&inp).unwrap();
                // TODO: Split the message in small parts, assign them to packets
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
        // println!("{:?}", msg);
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
            let _len = match stream.read_to_end(&mut buf) {
                Ok(l) => l,
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => break 'inner,
                Err(ref e) => panic!("{}", e),
            };

            let packet_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
            //println!("Length: {} (recv), {} (vec), {} (packet)", len, buf.len(), packet_len);
            //println!("Reply: {:?}", buf);
            // NOTE(feliix42): RFC 1035, 4.2.2 - TCP usage requires prepending the message with 2
            // bytes length information that does not include said two bytes
            let parsed = DNSMessage::from(&buf[2..packet_len + 2]);
            println!("{:#?}", parsed);
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
