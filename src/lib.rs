pub mod encoding;

use std::io::Read;
use std::io::{self, Seek};

use crate::encoding::ber_oid::read_ber_oid;

/// Reads the tag number from the buffer
///
/// Tag numbers are always stored in BER-OID format according to the `ST 0107.5
/// KLV Metadata in Motion Imagery` document.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER-OID
///
/// # Panics
///
/// - The value parsed from the BER-OID form won't fit in a u128.
pub fn read_tag_number<T>(buf: &mut T) -> Result<u128, io::Error>
where
    T: Read + Seek,
{
    read_ber_oid(buf)
}
