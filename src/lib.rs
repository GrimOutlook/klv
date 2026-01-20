use bitvec::vec::BitVec;
use byteorder::ReadBytesExt;
use std::io::Read;
use std::io::{self, Seek};

use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::view::BitView;

/// Reads the tag number from the buffer
///
/// Tag numbers are always stored in BER-OID format according to the `ST 0107.5
/// KLV Metadata in Motion Imagery` document.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER-OID
/// byte.
pub fn read_tag_number<T>(buf: &mut T) -> Result<u128, KlvParsingError>
where
    T: Read + Seek,
{
    read_ber_oid(buf)
}

/// Read in a BER-OID value from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER-OID
/// byte.
pub fn read_ber_oid<T>(buf: &mut T) -> Result<u128, KlvParsingError>
where
    T: Read + Seek,
{
    // Tag number should always start at the first byte.
    let mut bitvec = BitVec::<u8, Msb0>::new();
    loop {
        let byte = buf.read_u8()?;
        let bits = byte.view_bits::<Msb0>();
        bitvec.extend_from_bitslice(
            bits.get(1..bits.len())
                .expect("Cannot get bits after first for BER byte"),
        );
        // If the MSB is set then the Tag number is stored in BER format
        if !*bits.get(0).expect("Failed to get first bit from byte") {
            break;
        }
    }

    // Panic if the BER-OID bits make a number larger than can be represented in
    // a u128.
    bitvec = bitvec.drain(bitvec.leading_zeros()..bitvec.len()).collect();
    if bitvec.len() > 128 {
        panic!("BER-OID value was too large, with {} bits.", bitvec.len());
    }

    Ok(bitvec.load_be::<u128>())
}

#[derive(Debug, thiserror::Error)]
pub enum KlvParsingError {
    #[error("Other")]
    Other,
    #[error(transparent)]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{KlvParsingError, read_tag_number};
    use test_case::test_case;

    #[test_case(&[0x01], 1; "Smallest single-byte")]
    #[test_case(&[0x7F], 127; "Largest single-byte")]
    #[test_case(&[0x81, 0x00], 128; "Smallest two-byte")]
    #[test_case(&[0xFF, 0x7F], 16_383; "Largest two-byte")]
    #[test_case(&[0x83, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F], u128::MAX; "Largest representable")]
    fn read_tag_number_ok(input: &[u8], expected: u128) {
        assert_eq!(
            read_tag_number(&mut std::io::Cursor::new(input))
                .expect("Unexpected test case failure"),
            expected
        );
    }

    #[test_case(
        &[0x80],
        KlvParsingError::IoError(
            io::Error::from(io::ErrorKind::UnexpectedEof));
        "BER-OID ends with MSB set")]
    fn read_tag_number_err(input: &[u8], expected: KlvParsingError) {
        let err = read_tag_number(&mut std::io::Cursor::new(input))
            .expect_err("Testcase should fail here but does not");
        match err {
            KlvParsingError::IoError(inner) => {
                let KlvParsingError::IoError(expected_inner) = expected else {
                    panic!("Error's don't share the same type")
                };
                assert_eq!(inner.kind(), expected_inner.kind())
            }
            _ => assert_eq!(err.to_string(), expected.to_string()),
        }
    }
}
