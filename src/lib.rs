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
///
/// # Panics
///
/// - The value parsed from the BER-OID form won't fit in a u128.
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
///
/// # Panics
///
/// - The value parsed from the BER-OID form won't fit in a u128.
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

        if bitvec.len() == 7 {
            debug_assert!(
                bitvec.load_be::<u8>() != 0,
                "Multi-byte BER-OID starts with leading zero"
            );
        }
    }

    // Check to see if the bitvec only contains zeros, if it does then we can
    // just return zero.
    // NOTE: This is only needed because of how we strip the leading zeros
    // below.
    if bitvec.len() == bitvec.leading_zeros() {
        return Ok(0);
    }

    // Panic if the BER-OID bits make a number larger than can be represented in
    // a u128.
    bitvec = bitvec.drain(bitvec.leading_zeros()..bitvec.len()).collect();
    if bitvec.len() > 128 {
        panic!("BER-OID value was too large, with {} bits.", bitvec.len());
    }

    Ok(bitvec.load_be::<u128>())
}

/// Read in a BER value from the buffer.
///
/// Handles both BER short-form and BER long-form depending on the first bit of
/// the MSB.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER
/// byte.
///
/// # Panics
///
/// - The value parsed from the BER long form won't fit in a u128.
/// - The first bit is set but all other bits in the first byte are unset.
pub fn read_ber<T>(buf: &mut T) -> Result<u128, KlvParsingError>
where
    T: Read + Seek,
{
    let first_byte = buf.read_u8()?;
    let bits = first_byte.view_bits::<Msb0>();
    let value = if *bits.get(0).expect("Failed to get first bit from BER byte") {
        let num_bytes_to_read = bits
            .get(1..bits.len())
            .expect("Failed to read bits 1-7 for BER byte")
            .load_be();

        if num_bytes_to_read == 0 {
            panic!("MSB in BER is 1 but all other bits are 0");
        }

        read_ber_long_form(buf, num_bytes_to_read)?
    } else {
        first_byte as u128
    };

    Ok(value)
}

/// Read in a BER long-form value from the buffer using the number of bytes.
///
/// The first byte has already been read from the BER buffer in order to parse
/// the number of bytes.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER
/// byte.
///
/// # Panics
///
/// - The value parsed from the BER long form won't fit in a u128.
pub fn read_ber_long_form<T>(buf: &mut T, num_bytes_to_read: u8) -> Result<u128, KlvParsingError>
where
    T: Read + Seek,
{
    let mut bitvec = BitVec::<u8, Msb0>::new();
    for _ in 0..num_bytes_to_read {
        bitvec.extend_from_bitslice(buf.read_u8()?.view_bits::<Msb0>());
    }

    // Panic if the BER-OID bits make a number larger than can be represented in
    // a u128.
    bitvec = bitvec.drain(bitvec.leading_zeros()..bitvec.len()).collect();
    if bitvec.len() > 128 {
        panic!("BER value was too large, with {} bits.", bitvec.len());
    }
    let val = bitvec.load_be::<u128>();
    debug_assert!(val > 127, "BER long-form value could be stored short-form");
    Ok(val)
}

#[derive(Debug, thiserror::Error)]
pub enum KlvParsingError {
    #[error(transparent)]
    IoError(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{KlvParsingError, read_ber, read_ber_oid};
    use test_case::test_case;

    #[test_case(&[0x00], 0; "Zero")]
    #[test_case(&[0x01], 1; "Smallest single-byte")]
    #[test_case(&[0x7F], 127; "Largest single-byte")]
    #[test_case(&[0x81, 0x00], 128; "Smallest two-byte")]
    #[test_case(&[0xFF, 0x7F], 16_383; "Largest two-byte")]
    #[test_case(&[0x83, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F], u128::MAX; "Largest representable")]
    fn read_ber_oid_ok(input: &[u8], expected: u128) {
        assert_eq!(
            read_ber_oid(&mut std::io::Cursor::new(input)).expect("Unexpected test case failure"),
            expected
        );
    }

    #[test_case( &[], KlvParsingError::IoError( io::Error::from(io::ErrorKind::UnexpectedEof)); "BER-OID buffer has no bytes")]
    #[test_case( &[0x81], KlvParsingError::IoError( io::Error::from(io::ErrorKind::UnexpectedEof)); "BER-OID ends with MSB set")]
    fn read_ber_oid_err(input: &[u8], expected: KlvParsingError) {
        let err = read_ber_oid(&mut std::io::Cursor::new(input))
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

    #[should_panic]
    #[test_case(&[0x84, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00]; "Largest representable plus 1")]
    fn read_ber_oid_panics(input: &[u8]) {
        let _ = read_ber_oid(&mut std::io::Cursor::new(input));
    }

    #[test_case(&[0x00], 0; "Zero")]
    #[test_case(&[0x01], 1; "Smallest single-byte")]
    #[test_case(&[0x7F], 127; "Largest single-byte")]
    #[test_case(&[0x81, 0x80], 128; "Smallest two-byte")]
    #[test_case(&[0x90, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], u128::MAX; "Largest representable")]
    fn read_ber_ok(input: &[u8], expected: u128) {
        assert_eq!(
            read_ber(&mut std::io::Cursor::new(input)).expect("Unexpected test case failure"),
            expected
        );
    }

    #[test_case( &[], KlvParsingError::IoError( io::Error::from(io::ErrorKind::UnexpectedEof)); "BER buffer has no bytes")]
    #[test_case( &[0x81], KlvParsingError::IoError( io::Error::from(io::ErrorKind::UnexpectedEof)); "BER long-form ends after first byte")]
    fn read_ber_err(input: &[u8], expected: KlvParsingError) {
        let err = read_ber(&mut std::io::Cursor::new(input))
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

    #[should_panic]
    #[test_case(&[0x91, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; "Largest representable plus 1")]
    fn read_ber_panics(input: &[u8]) {
        let _ = read_ber(&mut std::io::Cursor::new(input));
    }
}
