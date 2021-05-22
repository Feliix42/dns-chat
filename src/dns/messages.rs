//! Data structures for DNS messages as defined in [RFC 1035](https://tools.ietf.org/html/rfc1035).
//!
//! This module implements data structures and methods for interacting with DNS messages, as far as necessary for the purpose of this application.

use super::types::*;
use std::convert::TryFrom;

/// A single DNS message.
///
/// This data structure is a simplified version of a DNS message, ignoring the `Authority` and `Additional` sections for resource records.
#[derive(Debug, PartialEq)]
pub struct DNSMessage {
    /// The DNS Header
    pub header: DNSHeader,
    /// Question section of a DNS message
    pub questions: Vec<DNSQuestion>,
    /// Answer section of the DNS message
    pub answers: Option<Vec<DNSAnswer>>,
}

impl DNSMessage {
    pub fn new_request(id: u16, domain: String) -> Self {
        let mut header = DNSHeader::new_request(id);

        let questions = vec![DNSQuestion::new_request(domain)];
        header.question_count += 1;

        DNSMessage {
            header,
            questions,
            answers: None,
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
            for s in question.name.split('.') {
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
                for s in answer.name.split('.') {
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
                    RecordData::Txt(contents) => {
                        for content in contents {
                            let bytes = content.as_bytes();
                            if bytes.len() > u8::MAX as usize {
                                // truncate sequence
                                msg.push(u8::MAX);
                                msg.extend_from_slice(&bytes[..u16::MAX as usize]);
                            } else {
                                msg.push(bytes.len() as u8);
                                msg.extend_from_slice(bytes);
                            }
                        }
                    }
                    // TODO(feliix42): This will probably produce a malformed DNS packet but I don't care right now
                    RecordData::Unsupported => (),
                }
            }
        }

        msg
    }
}

impl From<&[u8]> for DNSMessage {
    fn from(msg: &[u8]) -> Self {
        let header = DNSHeader::from(&msg[0..12]);

        println!(
            "Parsed header. {} questions and {} answers",
            header.question_count, header.answer_count
        );

        let mut pos = 12;
        let mut questions = Vec::with_capacity(header.question_count as usize);
        for _ in 0..header.question_count {
            let (question, new_pos) = DNSQuestion::parse(msg, pos);
            questions.push(question);
            pos = new_pos;
        }

        println!("Parsed questions");

        let answers = if header.answer_count > 0 {
            let mut answers = Vec::with_capacity(header.answer_count as usize);
            for _ in 0..header.answer_count {
                let (answer, new_pos) = DNSAnswer::parse(msg, pos);
                answers.push(answer);
                pos = new_pos;
            }
            Some(answers)
        } else {
            None
        };

        DNSMessage {
            header,
            questions,
            answers,
        }
    }
}

/// DNS header
#[derive(Debug, Default, PartialEq)]
pub struct DNSHeader {
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
    pub ar_count: u16,
}

impl DNSHeader {
    pub fn new_request(id: u16) -> Self {
        Self {
            id,
            recursion_desired: true,
            ..Default::default()
        }
    }
}

impl From<&[u8]> for DNSHeader {
    fn from(msg: &[u8]) -> Self {
        Self {
            id: u16::from_be_bytes([msg[0], msg[1]]),
            is_response: (msg[2] & 128u8) == 128,
            // bit mask for opcode: 01111000
            opcode: (msg[2] & 120u8) >> 3,
            authoritative_answer: (msg[2] & 4u8) == 4,
            is_truncated: (msg[2] & 2u8) == 2,
            recursion_desired: (msg[2] & 1u8) == 1,
            recursion_available: (msg[3] & 128u8) == 128,
            response_code: msg[3] & 15u8,
            question_count: u16::from_be_bytes([msg[4], msg[5]]),
            answer_count: u16::from_be_bytes([msg[6], msg[7]]),
            ns_record_count: u16::from_be_bytes([msg[8], msg[9]]),
            ar_count: u16::from_be_bytes([msg[10], msg[11]]),
        }
    }
}

/// A single DNS question
#[derive(Debug, PartialEq)]
pub struct DNSQuestion {
    /// Domain name in question.
    pub name: String,
    /// The type of record requested. Technically, the set of types supported here is just a subset of the types supported in RFC 1035, but supporting everything is not necessary.
    pub qtype: RecordType,
    /// The class of record requested.
    pub qclass: RecordClass,
}

impl DNSQuestion {
    fn parse(msg: &[u8], mut pos: usize) -> (DNSQuestion, usize) {
        let domain = parse_domain_name(msg, &mut pos);

        let qtype = RecordType::from(u16::from_be_bytes([msg[pos], msg[pos + 1]]));
        let qclass = RecordClass::from(u16::from_be_bytes([msg[pos + 2], msg[pos + 3]]));
        pos += 4;

        let question = DNSQuestion {
            name: domain,
            qtype,
            qclass,
        };

        (question, pos)
    }
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
#[derive(Debug, PartialEq)]
pub struct DNSAnswer {
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
    /// Record Data field
    pub record: RecordData,
}

impl DNSAnswer {
    fn parse(msg: &[u8], mut pos: usize) -> (DNSAnswer, usize) {
        let domain = parse_domain_name(msg, &mut pos);

        let rtype = RecordType::from(u16::from_be_bytes([msg[pos], msg[pos + 1]]));
        let rclass = RecordClass::from(u16::from_be_bytes([msg[pos + 2], msg[pos + 3]]));
        let ttl = u32::from_be_bytes([msg[pos + 4], msg[pos + 5], msg[pos + 6], msg[pos + 7]]);
        let data_length = u16::from_be_bytes([msg[pos + 8], msg[pos + 9]]);
        pos += 10;

        // parse the record data
        let record = match rtype {
            RecordType::TXT => {
                let mut contents = Vec::new();
                let mut total_len = 0;
                while total_len != data_length as usize {
                    let len = msg[pos] as usize;
                    pos += 1;
                    contents.push(String::from_utf8_lossy(&msg[pos..pos + len]).to_string());
                    total_len += 1 + len;
                    pos += len;
                }
                RecordData::Txt(contents)
            }
            _ => RecordData::Unsupported,
        };

        let answer = DNSAnswer {
            name: domain,
            rtype,
            rclass,
            ttl,
            data_length,
            record,
        };

        (answer, pos)
    }
}

fn parse_domain_name(msg: &[u8], pos: &mut usize) -> String {
    // check for compression, which is indicated by leading `11` in the first octet
    let is_backlink = msg[*pos] & 192u8 == 192;

    let mut domain = String::new();
    let mut first = true;
    let mut domain_pos = if is_backlink {
        // compression is enabled
        // mask the first two bits to get the position referenced
        (u16::from_be_bytes([msg[*pos], msg[*pos + 1]]) & 16383u16) as usize
    } else {
        *pos
    };

    while msg[domain_pos] != 0 {
        // append a dot in the domain name after the first sublabel
        if !first {
            domain.push('.');
        } else {
            first = false;
        }

        let len = dbg!(msg[domain_pos] as usize);
        println!("{}", is_backlink);

        domain.push_str(&String::from_utf8_lossy(
            &msg[(domain_pos + 1)..(domain_pos + len + 1)],
        ));

        domain_pos += len + 1;
    }
    println!("Finished parse: {}", domain);

    if is_backlink {
        // if it is a link, just increment by 2
        *pos += 2;
    } else {
        // if this was no link, set the pointer to the next element after the domain name
        *pos = domain_pos + 1;
    }

    domain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_to_u8_vec() {
        let expected = [
            91, 185, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 7, 101, 120, 97, 109, 112, 108, 101, 3, 99, 111,
            109, 0, 0, 16, 0, 1,
        ];

        let message = DNSMessage::new_request(23481, "example.com".into());
        let msg: Vec<u8> = message.into();

        assert_eq!(msg, expected);
    }

    #[test]
    fn conversion_from_u8_vec() {
        let input: Vec<u8> = vec![
            91, 185, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 7, 101, 120, 97, 109, 112, 108, 101, 3, 99, 111,
            109, 0, 0, 16, 0, 1,
        ];

        let expected = DNSMessage::new_request(23481, "example.com".into());

        let parsed = DNSMessage::from(input.as_slice());
        assert_eq!(parsed, expected);
    }
}
