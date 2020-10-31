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

impl Into<u16> for RecordType {
    fn into(self) -> u16 {
        match self {
            A => 1,
            NS => 2,
            MD => 3,
            MF => 4,
            CNAME => 5,
            SOA => 6,
            MB => 7,
            MG => 8,
            MR => 9,
            NULL => 10,
            WKS => 11,
            PTR => 12,
            HINFO => 13,
            MINFO => 14,
            MX => 15,
            TXT => 16
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

impl Into<u16> for RecordClass {
    fn into(self) -> u16 {
        match self {
            IN => 1,
            CS => 2,
            CH => 3,
            HS => 4
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
