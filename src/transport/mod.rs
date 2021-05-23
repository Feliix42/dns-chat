use crate::dns::types::RecordData;
use crate::dns::{messages::DNSMessage, types::RecordType};
use chrono::{DateTime, Local};

pub mod receiver;
pub mod sender;

/// The maximum length of a message per DNS message. This is the maximum number of bytes a TXT record can hold minus 25 bytes for the timestamp.
const MAX_MSG_LENGTH: usize = 65_254;

/// Representation of a single timestamped message
///
/// ## Maximum Length
/// Each message may only be up to 65,535 bytes long due to constraints in the DNS standard which requires that the length of the TXT record section (or any `RDATA` record for that matter) must be expressed by a unsigned 16 bit integer.
/// Internal formatting additionally requires a length byte every 255 bytes, reducing the effective maximum length to 65,279 bytes split into 256 blocks.
/// Additionally, 25 bytes are reserved for the message timestamp, bringing the length per message down to 65,254 bytes.
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub text: String,
    pub sent: DateTime<Local>,
}

impl ChatMessage {
    /// Converts a string into a series of timestamped chat messages.
    /// Should the length of the message exceed 65279 bytes, it is split into smaller chunks.
    pub fn from_str(mut msg: String) -> Vec<Self> {
        let timestamp = Local::now();

        // check if we need to split the message in several parts
        if msg.as_bytes().len() > MAX_MSG_LENGTH {
            // compute the # of necessary splits
            // TODO(feliix42): This could lead to an error when the message contains numerous badly aligned multi-byte characters and is sufficiently long. This would cause the string to be shifted to the right numerous times, outrunning the boudary calculated here. It's however very rare that this will happen.
            let msg_count = msg.as_bytes().len() / MAX_MSG_LENGTH
                + (msg.as_bytes().len() % MAX_MSG_LENGTH != 0) as usize;
            let mut messages = Vec::new();

            for _ in 0..(msg_count - 1) {
                let remainder = if msg.is_char_boundary(MAX_MSG_LENGTH) {
                    msg.split_off(MAX_MSG_LENGTH)
                } else {
                    msg.split_off(MAX_MSG_LENGTH - 1)
                };

                messages.push(Self {
                    text: msg,
                    sent: timestamp,
                });
                msg = remainder;
            }

            messages.push(Self {
                text: msg,
                sent: timestamp,
            });

            messages
        } else {
            vec![Self {
                text: msg,
                sent: timestamp,
            }]
        }
    }

    /// Converts a DNS message that was received into a vector of `ChatMessage` objects
    pub fn from_dns(dns_msg: DNSMessage) -> Vec<Self> {
        if let Some(mut answers) = dns_msg.answers {
            answers
                .drain(..)
                .filter(|m| m.rtype == RecordType::TXT)
                .map(|m| ChatMessage::from(m.record))
                .collect()
        } else {
            Vec::with_capacity(0)
        }
    }
}

impl Into<RecordData> for ChatMessage {
    fn into(self) -> RecordData {
        // append the time stamp to the message
        let mut msg_str = self.text;
        msg_str.insert_str(0, &self.sent.to_rfc3339());

        let mut strings = Vec::new();

        while !msg_str.is_empty() {
            let offset = if msg_str.len() < 255 {
                msg_str.len()
            } else if msg_str.is_char_boundary(255) {
                255
            } else {
                254
            };

            let remainder = msg_str.split_off(offset);
            strings.push(msg_str);
            msg_str = remainder;
        }

        RecordData::Txt(strings)
    }
}

impl From<RecordData> for ChatMessage {
    fn from(dns_msg: RecordData) -> Self {
        if let RecordData::Txt(mut strings) = dns_msg {
            let timestamp = DateTime::from(
                DateTime::parse_from_rfc3339(&strings[0][0..25])
                    .expect("Sender delivered malformed date information"),
            );
            strings[0] = strings[0].split_off(25);

            let mut msg = String::with_capacity(strings.iter().fold(0, |acc, s| acc + s.len()));
            for s in &strings {
                msg.push_str(s);
            }

            Self {
                text: msg,
                sent: timestamp,
            }
        } else {
            panic!("Non-text RecordData type")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_to_record() {
        let msg = ChatMessage {
            text: String::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            sent: DateTime::from(DateTime::parse_from_rfc3339("2020-12-24T18:34:16+01:00").unwrap())
        };

        let expected = RecordData::Txt(vec![
            String::from("2020-12-24T18:34:16+01:00aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            String::from("aaaaaaaaaaaaaaaaaaaaaaaaa")
        ]);

        let res: RecordData = msg.into();
        assert_eq!(res, expected);
    }
}
