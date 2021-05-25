use clap::Clap;
use opts::Opts;
use std::sync::mpsc;
use std::thread;

mod dns;
mod opts;
mod state;
mod transport;
mod tui;

fn main() -> std::io::Result<()> {
    let Opts {
        target,
        target_port,
        listening_port,
    } = Opts::parse();

    let (msg_sender, rx) = mpsc::channel();
    let sender = thread::Builder::new().name("Sender".to_string());
    sender
        .spawn(move || transport::sender::run_sender(rx, listening_port))
        .expect("Could not spawn sender thread");

    let (sx, msg_recv) = mpsc::channel();
    let receiver = thread::Builder::new().name("Receiver".to_string());
    receiver
        .spawn(move || transport::receiver::poll_messages(sx, (target, target_port)))
        .expect("Could not spawn receiver thread");

    if let Err(e) = tui::run(msg_sender, msg_recv) {
        eprintln!("{}", e);
    }

    Ok(())
}
