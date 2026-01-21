use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::view::BitView;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use std::io::Seek;

use std::io::Read;

use crate::encoding::Error;

/// Integer types that can be read in using `read_integer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Integer {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

/// Read in a variable length integer.
///
/// Integers can be stored in variable lengths that adjust based on their
/// numeric values. Successfully parsed integers are always returned in the
/// smallest datatype the number of bytes read in fits in (e.g., 3 bytes will
/// always return an `i32`).
///
/// # Args
///
/// - `buf` - Buffer to read from.
/// - `length` - Number of bytes to read and interpret as an integer.
///
/// # Returns
///
/// - `Ok(Integer)` - Number of bytes read in from buffer can successfully be
///   interpreted as an i128 or smaller datatype.
/// - `Err(encoding::Error)` - Number of bytes read cannot fit into an integer
///   container, number of bytes to read is zero, or there is an error reading
///   from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_integer<T>(buf: &mut T, length: u8) -> Result<Integer, Error>
where
    T: Read + Seek,
{
    let value = match length {
        1 => Integer::I8(buf.read_i8()?),
        2 => Integer::I16(buf.read_i16::<byteorder::BigEndian>()?),
        3 | 4 => Integer::I32(
            buf.read_int::<byteorder::BigEndian>(length as usize)?
                .try_into()
                .unwrap_or_else(|_| panic!("{length} bytes doesn't fit in an `i32` somehow")),
        ),
        5..=8 => Integer::I64(buf.read_int::<byteorder::BigEndian>(length as usize)?),
        9..=16 => {
            let bytes = (0..length)
                .map(|_| buf.read_u8().map_err(Error::from))
                .collect::<Result<Vec<u8>, Error>>()?;
            let mut bits_mut = bytes.view_bits::<Msb0>().to_bitvec();
            let initial_bits = bits_mut.clone();
            let is_negative = initial_bits.first().unwrap();
            // If the first bit is set then this represents a negative number
            // and the MSB must be moved when growing the number to the full
            // length of an i128
            if *is_negative {
                *bits_mut.first_mut().unwrap() = false
            }
            // Insert 0s until the bitvec is the same size as a normal i128
            ((length * 8) as u32..i128::BITS).for_each(|_| bits_mut.insert(0, false));
            if *is_negative {
                *bits_mut.first_mut().unwrap() = true
            }

            Integer::I128(bits_mut.load_be())
        }
        _ => return Err(Error::DecodingError("integer".to_string())),
    };

    Ok(value)
}

/// Reads 1 byte and interprets it as na `i8`.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_i8` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(i8)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_i8<T>(buf: &mut T) -> Result<i8, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_i8()?)
}

/// Reads 2 bytes and interpresets it as an `i16` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_i16` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(i16)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_i16<T>(buf: &mut T) -> Result<i16, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_i16::<BigEndian>()?)
}

/// Reads 4 bytes and interpresets it as an `i32` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_i32` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(i16)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_i32<T>(buf: &mut T) -> Result<i32, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_i32::<BigEndian>()?)
}

/// Reads 8 bytes and interpresets it as an `i64` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_i64` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(i64)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_i64<T>(buf: &mut T) -> Result<i64, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_i64::<BigEndian>()?)
}

/// Reads 16 bytes and interpresets it as an `i128` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_i128` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(i128)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_i128<T>(buf: &mut T) -> Result<i128, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_i128::<BigEndian>()?)
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use test_case::test_case;

    #[test_case(&[0x00], Integer::I8(0); "i8 Zero")]
    #[test_case(&[0x80], Integer::I8(i8::MIN); "i8 Min")]
    #[test_case(&[0x7F], Integer::I8(i8::MAX); "i8 Max")]
    #[test_case(&[0x00, 0x00], Integer::I16(0); "i16 Zero")]
    #[test_case(&[0x80, 0x00], Integer::I16(i16::MIN); "i16 Min")]
    #[test_case(&[0x7F, 0xFF], Integer::I16(i16::MAX); "i16 Max")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00], Integer::I32(0); "i32 Zero")]
    #[test_case(&[0x80, 0x00, 0x00, 0x00], Integer::I32(i32::MIN); "i32 Min")]
    #[test_case(&[0x7F, 0xFF, 0xFF, 0xFF], Integer::I32(i32::MAX); "i32 Max")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Integer::I64(0); "i64 Zero")]
    #[test_case(&[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Integer::I64(i64::MIN); "i64 Min")]
    #[test_case(&[0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], Integer::I64(i64::MAX); "i64 Max")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Integer::I128(0); "i128 Zero")]
    #[test_case(&[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Integer::I128(i128::MIN); "i128 Min")]
    #[test_case(&[0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], Integer::I128(i128::MAX); "i128 Max")]
    fn read_integer_ok(input: &[u8], expected: Integer) {
        assert_eq!(
            read_integer(
                &mut std::io::Cursor::new(input),
                input.len().try_into().unwrap()
            )
            .expect("Unexpected test case failure"),
            expected
        );
    }

    #[test_case( &[], io::Error::from(io::ErrorKind::UnexpectedEof); "Integer buffer has no bytes")]
    #[test_case( &[0x81], io::Error::from(io::ErrorKind::UnexpectedEof); "Integer buffer ends unexpectedly")]
    fn read_integer_io_err(input: &[u8], expected: io::Error) {
        let err = read_integer(&mut std::io::Cursor::new(input), 2)
            .expect_err("Testcase should fail here but does not")
            .try_as_other()
            .unwrap();
        assert_eq!(err.kind(), expected.kind())
    }

    #[test_case( &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F], 17, Error::DecodingError("integer".to_string()); "Length too long")]
    #[test_case( &[0x00], 0, Error::DecodingError("integer".to_string()); "Length to read is zero")]
    fn read_integer_decoding_err(input: &[u8], length: u8, expected: Error) {
        let err = read_integer(&mut std::io::Cursor::new(input), length)
            .expect_err("Testcase should fail here but does not");
        assert_eq!(err.to_string(), expected.to_string())
    }
}
