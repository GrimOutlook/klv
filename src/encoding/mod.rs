pub mod ber;
pub mod ber_oid;

pub trait EnumerationValue {}

/// Values enumerated here are copied from _Table 40_ on page 115 of
/// _MISP-2025.1: Motion Imagery Handbook_
pub enum SimpleDataType {
    Ber,
    BerOid,
    Binary,
    Boolean,
    Iso7,
    Utf8,
    Utf16,
    Enumeration,
    FloatingPoint,

    /// The IMAP type is an unsigned integer, which is a mapping to a
    /// floating-point value as specified by MISB ST 1201. Knowing certain
    /// parameters (min, max, resolution) about the value enables this
    /// representation to use fewer bytes than an equivalent IEEE 754
    /// floating-point value
    IMAP,

    Integer,
    UnsignedInteger,
}
