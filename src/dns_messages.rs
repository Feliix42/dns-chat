//! Data structures for DNS messages as defined in [RFC 1035](https://tools.ietf.org/html/rfc1035).
//!
//! This module implements data structures and methods for interacting with DNS messages, as far as necessary for the purpose of this application.

use std::convert::TryFrom;
use crate::types::*;

/// A single DNS message.
/// 
/// This data structure is a simplified version of a DNS message, ignoring the `Authority` and `Additional` sections for resource records.
pub struct DNSMessage {
    /// The DNS Header
    header: DNSHeader,
    /// Question section of a DNS message
    questions: Vec<DNSQuestion>,
    /// Answer section of the DNS message
    answers: Option<Vec<DNSAnswer>>,
}

impl DNSMessage {
    pub fn new_request(id: u16, domain: String) -> Self {
        let mut header = DNSHeader::new_request(id);

        let questions = vec![DNSQuestion::new_request(domain)];
        header.question_count += 1;

        DNSMessage {
            header,
            questions,
            answers: None
        }
    }
}

impl Into<Vec<u8>> for DNSMessage {
    fn into(self) -> Vec<u8> {
        let mut msg = Vec::with_capacity(12);

        // header processing
        let header = self.header;
        msg.extend_from_slice(&header.id.to_be_bytes());

        let mut byte_3: u8 = u8::from(header.is_response) << 7;
        byte_3 += header.opcode << 3;
        byte_3 += u8::from(header.authoritative_answer) << 2;
        byte_3 += u8::from(header.is_truncated) << 1;
        byte_3 += u8::from(header.recursion_desired);
        msg.push(byte_3);

        let mut byte_4: u8 = u8::from(header.recursion_available) << 7;
        byte_4 += header.response_code;
        msg.push(byte_4);

        msg.extend_from_slice(&header.question_count.to_be_bytes());
        msg.extend_from_slice(&header.answer_count.to_be_bytes());
        msg.extend_from_slice(&header.ns_record_count.to_be_bytes());
        msg.extend_from_slice(&header.ar_count.to_be_bytes());

        // question section processing
        let questions = self.questions;
        for question in questions {
            for s in question.name.split(".") {
                let bytes = s.as_bytes();
                msg.push(u8::try_from(bytes.len()).unwrap());
                msg.extend_from_slice(bytes);
            }
            msg.push(0);

            msg.extend_from_slice(&u16::from(question.qtype).to_be_bytes());
            msg.extend_from_slice(&u16::from(question.qclass).to_be_bytes());
        }

        // answer section processing
        if let Some(answers) = self.answers {
            for answer in answers {
                for s in answer.name.split(".") {
                    let bytes = s.as_bytes();
                    msg.push(u8::try_from(bytes.len()).unwrap());
                    msg.extend_from_slice(bytes);
                }
                msg.push(0);

                msg.extend_from_slice(&u16::from(answer.rtype).to_be_bytes());
                msg.extend_from_slice(&u16::from(answer.rclass).to_be_bytes());
                msg.extend_from_slice(&answer.ttl.to_be_bytes());
                msg.extend_from_slice(&answer.data_length.to_be_bytes());

                match answer.record {
                    RecordData::Txt(content) => {
                        let bytes = content.as_bytes();
                        if bytes.len() > u16::MAX as usize {
                            #[cold]
                            // truncate sequence
                            msg.extend_from_slice(&bytes[..u16::MAX as usize]);
                        } else {
                            msg.extend_from_slice(bytes);
                        }
                    },
                    // TODO(feliix42): This will probably produce a malformed DNS packet but I don't care right now
                    RecordData::Unsupported => (),
                }
            }
        }

        msg
    }
}

/// DNS header
#[derive(Default)]
struct DNSHeader {
    /// A unique ID for referencing the request
    pub id: u16,
    /// Described whether the message is a question or a reply
    pub is_response: bool,
    /// The message opcode describing the kind of query
    pub opcode: u8,
    /// Is the name server an authority for the domain name in question?
    pub authoritative_answer: bool,
    /// Whether the message was truncated
    pub is_truncated: bool,
    /// Allows the name server to employ recursion if set to `true`.
    pub recursion_desired: bool,
    /// Described whether the name server answering can offer recursive lookup.
    pub recursion_available: bool,
    /// The response error code of a message
    pub response_code: u8,
    /// Number of questions following this header
    pub question_count: u16,
    /// Number of answers following this header
    pub answer_count: u16,
    /// Number of name server resource records in the authority records
    pub ns_record_count: u16,
    /// Number of resource records in the additional records section
    pub ar_count: u16
}

impl DNSHeader {
    pub fn new_request(id: u16) -> Self {
        let mut header = Self::default();

        header.id = id;
        header.recursion_desired = true;

        header
    }
}

/// A single DNS question
struct DNSQuestion {
    /// Domain name in question.
    pub name: String,
    /// The type of record requested. Technically, the set of types supported here is just a subset of the types supported in RFC 1035, but supporting everything is not necessary.
    pub qtype: RecordType,
    /// The class of record requested.
    pub qclass: RecordClass
}

impl DNSQuestion {
    pub fn new_request(domain: String) -> Self {
        DNSQuestion {
            name: domain,
            qtype: RecordType::TXT,
            qclass: RecordClass::IN,
        }
    }
}

/// A single reply to a DNS query.
struct DNSAnswer {
    /// Domain name in question.
    pub name: String,
    /// The type of the transmitted record
    pub rtype: RecordType,
    /// Class of the transmitted record
    pub rclass: RecordClass,
    /// Time to live of the record. `0` indicates that this response should not be cached.
    pub ttl: u32,
    /// Length of the record data field
    pub data_length: u16,
    // TODO(feliix42): Make the max length of a record data field obvious somewhere
    /// Record Data field
    pub record: RecordData,
}
