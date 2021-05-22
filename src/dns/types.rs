//! A simple collection of DNS types, tailored towards the use in this application.
//!
//! Since only a small fraction of the whole DNS specification is needed for this application, not everything has been implemented.

/// Type Fields used in Reqource records and also in questions.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
pub enum RecordType {
    /// an IPv4 host address
    A,
    /// an authoritative name server
    NS,
    /// mail destination (Obsolete - use MX)
    MD,
    /// mail forwarder (Obsolete - use MX)
    MF,
    /// canonical name for an alias
    CNAME,
    /// start of a zone of authority
    SOA,
    /// mailbox domain name _(experimental)_
    MB,
    /// mail group member _(experimental)_
    MG,
    /// mail rename domain name _(experimental)_
    MR,
    /// a null RR _(experimental)_
    NULL,
    /// well known service description
    WKS,
    /// domain name pointer
    PTR,
    /// host information
    HINFO,
    /// mailbox or mail list information
    MINFO,
    /// mail exchange
    MX,
    /// text string
    TXT,
}

impl From<RecordType> for u16 {
    fn from(data: RecordType) -> Self {
        match data {
            RecordType::A => 1,
            RecordType::NS => 2,
            RecordType::MD => 3,
            RecordType::MF => 4,
            RecordType::CNAME => 5,
            RecordType::SOA => 6,
            RecordType::MB => 7,
            RecordType::MG => 8,
            RecordType::MR => 9,
            RecordType::NULL => 10,
            RecordType::WKS => 11,
            RecordType::PTR => 12,
            RecordType::HINFO => 13,
            RecordType::MINFO => 14,
            RecordType::MX => 15,
            RecordType::TXT => 16,
        }
    }
}

impl From<u16> for RecordType {
    fn from(data: u16) -> Self {
        match data {
            1 => RecordType::A,
            2 => RecordType::NS,
            3 => RecordType::MD,
            4 => RecordType::MF,
            5 => RecordType::CNAME,
            6 => RecordType::SOA,
            7 => RecordType::MB,
            8 => RecordType::MG,
            9 => RecordType::MR,
            10 => RecordType::NULL,
            11 => RecordType::WKS,
            12 => RecordType::PTR,
            13 => RecordType::HINFO,
            14 => RecordType::MINFO,
            15 => RecordType::MX,
            16 => RecordType::TXT,
            _ => panic!("Unsupported Record Type {}", data),
        }
    }
}

/// The class of a resource record
#[derive(Debug, PartialEq)]
pub enum RecordClass {
    /// The Internet
    IN,
    /// The CSNET class (obsolete)
    CS,
    /// the CHAOS clas
    CH,
    /// Hesoid
    HS,
}

impl From<RecordClass> for u16 {
    fn from(data: RecordClass) -> Self {
        match data {
            RecordClass::IN => 1,
            RecordClass::CS => 2,
            RecordClass::CH => 3,
            RecordClass::HS => 4,
        }
    }
}

impl From<u16> for RecordClass {
    fn from(data: u16) -> Self {
        match data {
            1 => RecordClass::IN,
            2 => RecordClass::CS,
            3 => RecordClass::CH,
            4 => RecordClass::HS,
            _ => panic!("Unsupported Record Class {:?}", data.to_be_bytes()),
        }
    }
}

/// The RDATA field of a resource record. May not exceed 65,535 Bytes.
#[derive(Debug, PartialEq)]
pub enum RecordData {
    /// A TXT record. One String may not be longer than 255 bytes.
    Txt(Vec<String>),
    /// Not supported record type
    Unsupported,
}
