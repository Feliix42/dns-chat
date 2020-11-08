use chrono::{DateTime, Local};
use crate::dns::messages::DNSMessage;
use std::fmt;

pub mod sender;
pub mod receiver;

/// Representation of a single timestamped message
/// 
/// Each Message may only be up to TODO bytes long due to constraints in the DNS standard.
#[derive(Debug)]
pub struct ChatMessage {
    text: String,
    sent: DateTime<Local>,
}

impl ChatMessage {
    pub fn from_str(msg: String) -> Vec<Self> {
        unimplemented!()
    }
}

impl Into<DNSMessage> for ChatMessage {
    fn into(self) -> DNSMessage {
        unimplemented!()
    }
}

impl From<DNSMessage> for ChatMessage {
    fn from(dns_msg: DNSMessage) -> Self {
        unimplemented!()
    }
}

impl fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
        // write!(f, "")
    }
}
