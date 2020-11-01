//! A simple collection of DNS types, tailored towards the use in this application.
//! 
//! Since only a small fraction of the whole DNS specification is needed for this application, not everything has been implemented.

/// Type Fields used in Reqource records and also in questions.
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
    TXT
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
            RecordType::TXT => 16
        }
    }
}

/// The class of a resource record
pub enum RecordClass {
    /// The Internet
    IN,
    /// The CSNET class (obsolete)
    CS,
    /// the CHAOS clas
    CH,
    /// Hesoid
    HS
}

impl From<RecordClass> for u16 {
    fn from(data: RecordClass) -> Self {
        match data {
            RecordClass::IN => 1,
            RecordClass::CS => 2,
            RecordClass::CH => 3,
            RecordClass::HS => 4
        }
    }
}

/// The RDTA field of a resource record. May not exceed 65,535 Bytes.
pub enum RecordData {
    /// A TXT record
    Txt(String),
    /// Not supported record type
    Unsupported
}
