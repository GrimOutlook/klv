pub mod encoding;
pub mod klv;
pub mod local_set;
pub mod universal_set;

use std::io::Read;
use std::io::{self, Seek};

use crate::encoding::ber::read_ber;
use crate::encoding::ber_oid::read_ber_oid;

/// Reads the tag number from the buffer
///
/// Tag numbers are always stored in BER-OID format according to the `ST 0107.5
/// KLV Metadata in Motion Imagery` document section `6.3.1`.
///
/// # Returns
///
/// - Ok(u128) - When a valid u128 BER-OID value can be read from the given buffer.
/// - Err(std::io::Error) - When a valid u128 BER-OID value cannot be read from the given buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER-OID
/// byte.
///
/// # Panics
///
/// - The value parsed from the BER-OID won't fit in a u128.
pub fn read_tag_number<T>(buf: &mut T) -> Result<u128, io::Error>
where
    T: Read + Seek,
{
    read_ber_oid(buf)
}

/// Reads the length of the KLV value from the buffer
///
/// Value lengthjs are always stored in BER format according to the `ST 0107.5
/// KLV Metadata in Motion Imagery` document section `6.3.2`.
///
/// # Returns
///
/// - Ok(u128) - When a valid u128 BER value can be read from the given buffer.
/// - Err(std::io::Error) - When a valid u128 BER value cannot be read from the given buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER
/// byte.
///
/// # Panics
///
/// - The value parsed from the BER is long-form and won't fit in a u128.
pub fn read_length<T>(buf: &mut T) -> Result<u128, io::Error>
where
    T: Read + Seek,
{
    read_ber(buf)
}
