use std::io;

use crate::encoding::{integer::SignedInteger, unsigned_integer::UnsignedInteger};

pub mod ber;
pub mod ber_oid;
pub mod integer;
pub mod unsigned_integer;

/// Values enumerated here are copied from _Table 40_ on page 115 of
/// _MISP-2025.1: Motion Imagery Handbook_
#[derive(Clone, Debug, strum::EnumDiscriminants)]
pub enum SimpleDataType {
    Ber(u128),
    BerOid(u128),

    /// A Binary data type compacts a set of information into one or more bytes.
    /// The binary type can contain flags, small enumerations and other
    /// bit-specified controls. When MISP metadata standards use binary types,
    /// the standard describes how to interpret each bit of the binary value
    Binary(Vec<u8>),

    /// With a byte, when the byte value equals zero (0x00), the Boolean meaning
    /// is false; when the byte value is one (0x01) the Boolean meaning it true;
    /// all other values are not valid.
    Boolean(bool),

    /// The ISO7 data type is one or more characters from the ISO/IEC 646:1991
    /// [19] character set. ISO7 uses the lower seven bits of a byte; the eighth
    /// bit is always zero.
    ///
    /// WARN: ISO7 supports the Latin character set for English only. For this
    /// reason, the MISP is discontinuing ISO7 in document updates and new
    /// publications. Some existing MISP standards use ISO7 for historical and
    /// backward compatibility reasons. The replacement data type for ISO7 is
    /// UTF8.
    Iso7(String),

    /// MISP standards use UTF8 for character strings because it expands as
    /// necessary to support alternate languages in support of NATO countries.
    Utf8(String),

    /// WARN: Older MISP standards have used UTF16, but all document updates and
    /// new publications will utilize UTF8 instead of UTF16.
    Utf16,

    /// An unsigned integer whose value maps to a predefined table of choices. A
    /// controlling document (e.g., standard) defines the range of allowed
    /// unsigned integer values beginning at zero to some maximum value along
    /// with the meanings for each value. An enumeration compacts a list of
    /// choices into a single unsigned integer value, thereby saving bytes.
    Enumeration(u128),

    FloatingPoint,

    /// The IMAP type is an unsigned integer, which is a mapping to a
    /// floating-point value as specified by MISB ST 1201. Knowing certain
    /// parameters (min, max, resolution) about the value enables this
    /// representation to use fewer bytes than an equivalent IEEE 754
    /// floating-point value
    IMAP,

    SignedInteger(SignedInteger),
    UnsignedInteger(UnsignedInteger),
}

#[derive(Debug, strum::EnumTryAs, thiserror::Error)]
pub enum Error {
    #[error("Failed to decode {0}")]
    DecodingError(String),
    #[error(transparent)]
    Other(#[from] io::Error),
}
