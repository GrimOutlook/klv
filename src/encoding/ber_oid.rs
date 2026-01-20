use bitvec::field::BitField;
use bitvec::prelude::BitVec;
use bitvec::prelude::Msb0;
use bitvec::view::BitView;
use byteorder::ReadBytesExt;
use std::io;
use std::io::Read;
use std::io::Seek;

/// Read in a BER-OID value from the buffer.
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
/// - The value parsed from the BER-OID form won't fit in a u128.
pub fn read_ber_oid<T>(buf: &mut T) -> Result<u128, io::Error>
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

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
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

    #[test_case( &[], io::Error::from(io::ErrorKind::UnexpectedEof); "BER-OID buffer has no bytes")]
    #[test_case( &[0x81], io::Error::from(io::ErrorKind::UnexpectedEof); "BER-OID ends with MSB set")]
    fn read_ber_oid_err(input: &[u8], expected: io::Error) {
        let err = read_ber_oid(&mut std::io::Cursor::new(input))
            .expect_err("Testcase should fail here but does not");
        assert_eq!(err.kind(), expected.kind())
    }

    #[should_panic]
    #[test_case(&[0x84, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00]; "Largest representable plus 1")]
    fn read_ber_oid_panics(input: &[u8]) {
        let _ = read_ber_oid(&mut std::io::Cursor::new(input));
    }
}
