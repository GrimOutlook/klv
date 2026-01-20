use bitvec::field::BitField;
use bitvec::prelude::BitVec;
use bitvec::prelude::Msb0;
use bitvec::view::BitView;
use byteorder::ReadBytesExt;
use std::io;
use std::io::Read;
use std::io::Seek;

/// Read in a BER value from the buffer.
///
/// Handles both BER short-form and BER long-form depending on the first bit of
/// the MSB.
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
/// - The first bit is set but all other bits in the first byte are unset.
pub fn read_ber<T>(buf: &mut T) -> Result<u128, io::Error>
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
/// # Returns
///
/// - Ok(u128) - When a valid u128 BER long-form value can be read from the given buffer.
/// - Err(std::io::Error) - When a valid u128 BER long-form value cannot be read from the given buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last BER
/// byte.
///
/// # Panics
///
/// - The value parsed from the BER long form won't fit in a u128.
pub fn read_ber_long_form<T>(buf: &mut T, num_bytes_to_read: u8) -> Result<u128, io::Error>
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

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use test_case::test_case;

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

    #[test_case( &[], io::Error::from(io::ErrorKind::UnexpectedEof); "BER buffer has no bytes")]
    #[test_case( &[0x81], io::Error::from(io::ErrorKind::UnexpectedEof); "BER long-form ends after first byte")]
    fn read_ber_err(input: &[u8], expected: io::Error) {
        let err = read_ber(&mut std::io::Cursor::new(input))
            .expect_err("Testcase should fail here but does not");
        assert_eq!(err.kind(), expected.kind())
    }

    #[should_panic]
    #[test_case(&[0x91, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; "Largest representable plus 1")]
    fn read_ber_panics(input: &[u8]) {
        let _ = read_ber(&mut std::io::Cursor::new(input));
    }
}
