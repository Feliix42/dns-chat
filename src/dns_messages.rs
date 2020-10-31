//! Data structures for DNS messages as defined in [RFC 1035](https://tools.ietf.org/html/rfc1035).
//!
//! This module implements data structures and methods for interacting with DNS messages, as far as necessary for the purpose of this application.

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

/// DNS header
struct DNSHeader {
    /// A unique ID for referencing the request
    id: u16,
    /// Described whether the message is a question or a reply
    is_response: bool,
    /// The message opcode describing the kind of query
    opcode: u8,
    /// Is the name server an authority for the domain name in question?
    authoritative_answer: bool,
    /// Whether the message was truncated
    is_truncated: bool,
    /// Allows the name server to employ recursion if set to `true`.
    recursion_desired: bool,
    /// Described whether the name server answering can offer recursive lookup.
    recursion_available: bool,
    /// The response error code of a message
    response_code: u8,
    /// Number of questions following this header
    question_count: u16,
    /// Number of answers following this header
    answer_count: u16,
    /// Number of name server resource records in the authority records
    ns_record_count: u16,
    /// Number of resource records in the additional records section
    ar_count: u16
}

/// A single DNS question
struct DNSQuestion {
    /// Domain name in question.
    name: String,
    /// The type of record requested. Technically, the set of types supported here is just a subset of the types supported in RFC 1035, but supporting everything is not necessary.
    qtype: RecordType,
    /// The class of record requested.
    qclass: RecordClass
}

/// A single reply to a DNS query.
struct DNSAnswer {
    /// Domain name in question.
    name: String,
    /// The type of the transmitted record
    rtype: RecordType,
    /// Class of the transmitted record
    rclass: RecordClass,
    /// Time to live of the record. `0` indicates that this response should not be cached.
    ttl: u32,
    /// Length of the record data field
    data_length: u16,
    /// Record Data field
    record: RecordData,
}
