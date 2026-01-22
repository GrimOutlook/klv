use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::view::BitView;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;
use std::io::Seek;

use std::io::Read;

use crate::encoding::Error;

/// UnsignedInteger types that can be read in using `read_unsigned_integer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsignedInteger {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

/// Read in a variable length unsigned integer.
///
/// Unsigned integers can be stored in variable lengths that adjust based on
/// their numeric values. Successfully parsed unsigned integers are always
/// returned in the smallest datatype the number of bytes read in fits in (e.g.,
/// 3 bytes will always return a `u32`).
///
/// # Args
///
/// - `buf` - Buffer to read from.
/// - `length` - Number of bytes to read and interpret as an unsigned integer.
///
/// # Returns
///
/// - `Ok(UnsignedInteger)` - Number of bytes read in from buffer can
///   successfully be interpreted as an u128 or smaller datatype.
/// - `Err(encoding::Error)` - Number of bytes read cannot fit into an unsigned
///   integer container, number of bytes to read is zero, or there is an error
///   reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_unsigned_integer<T>(buf: &mut T, length: u8) -> Result<UnsignedInteger, Error>
where
    T: Read + Seek,
{
    let value = match length {
        1 => UnsignedInteger::U8(buf.read_u8()?),
        2 => UnsignedInteger::U16(buf.read_u16::<byteorder::BigEndian>()?),
        3 | 4 => UnsignedInteger::U32(
            buf.read_uint::<byteorder::BigEndian>(length as usize)?
                .try_into()
                .unwrap_or_else(|_| panic!("{length} bytes doesn't fit in a `u32` somehow")),
        ),
        5..=8 => UnsignedInteger::U64(buf.read_uint::<byteorder::BigEndian>(length as usize)?),
        9..=16 => {
            let bytes = (0..length)
                .map(|_| buf.read_u8().map_err(Error::from))
                .collect::<Result<Vec<u8>, Error>>()?;
            UnsignedInteger::U128(bytes.view_bits::<Msb0>().load_be())
        }
        _ => return Err(Error::DecodingError("unsigned_integer".to_string())),
    };

    Ok(value)
}

/// Reads 1 byte and interprets it as na `u8`.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_u8` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(u8)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_u8<T>(buf: &mut T) -> Result<u8, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_u8()?)
}

/// Reads 2 bytes and interpresets it as an `u16` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_u16` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(u16)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_u16<T>(buf: &mut T) -> Result<u16, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_u16::<BigEndian>()?)
}

/// Reads 4 bytes and interpresets it as an `u32` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_u32` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(u16)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_u32<T>(buf: &mut T) -> Result<u32, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_u32::<BigEndian>()?)
}

/// Reads 8 bytes and interpresets it as an `u64` in `BigEndian` format.
///
/// This is just a wrapper around `byteorder::ReadBytesExt::read_u64` provided
/// for convenience.
///
/// # Returns
///
/// - `Ok(u64)` - Data was read without error.
/// - `Err(encoding::Error)` - There was an error reading from the buffer.
///
/// # Side Effects
///
/// Moves the current position in the buffer to the byte after the last byte
/// read
pub fn read_u64<T>(buf: &mut T) -> Result<u64, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_u64::<BigEndian>()?)
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
pub fn read_u128<T>(buf: &mut T) -> Result<u128, Error>
where
    T: Read + Seek,
{
    Ok(buf.read_u128::<BigEndian>()?)
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use test_case::test_case;

    #[test_case(&[0x00], UnsignedInteger::U8(0); "u8 Zero")]
    #[test_case(&[0x00], UnsignedInteger::U8(u8::MIN); "u8 Min")]
    #[test_case(&[0xFF], UnsignedInteger::U8(u8::MAX); "u8 Max")]
    #[test_case(&[0x00, 0x00], UnsignedInteger::U16(0); "u16 Zero")]
    #[test_case(&[0x00, 0x00], UnsignedInteger::U16(u16::MIN); "u16 Min")]
    #[test_case(&[0xFF, 0xFF], UnsignedInteger::U16(u16::MAX); "u16 Max")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00], UnsignedInteger::U32(0); "u32 Zero")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00], UnsignedInteger::U32(u32::MIN); "u32 Min")]
    #[test_case(&[0xFF, 0xFF, 0xFF, 0xFF], UnsignedInteger::U32(u32::MAX); "u32 Max")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], UnsignedInteger::U64(0); "u64 Zero")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], UnsignedInteger::U64(u64::MIN); "u64 Min")]
    #[test_case(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], UnsignedInteger::U64(u64::MAX); "u64 Max")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], UnsignedInteger::U128(0); "u128 Zero")]
    #[test_case(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], UnsignedInteger::U128(u128::MIN); "u128 Min")]
    #[test_case(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], UnsignedInteger::U128(u128::MAX); "u128 Max")]
    fn read_unsigned_integer_ok(input: &[u8], expected: UnsignedInteger) {
        assert_eq!(
            read_unsigned_integer(
                &mut std::io::Cursor::new(input),
                input.len().try_into().unwrap()
            )
            .expect("Unexpected test case failure"),
            expected
        );
    }

    #[test_case( &[], io::Error::from(io::ErrorKind::UnexpectedEof); "Unsigned Integer buffer has no bytes")]
    #[test_case( &[0x81], io::Error::from(io::ErrorKind::UnexpectedEof); "Unsigned Integer buffer ends unexpectedly")]
    fn read_integer_io_err(input: &[u8], expected: io::Error) {
        let err = read_unsigned_integer(&mut std::io::Cursor::new(input), 2)
            .expect_err("Testcase should fail here but does not")
            .try_as_other()
            .unwrap();
        assert_eq!(err.kind(), expected.kind())
    }

    #[test_case( &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F], 17, Error::DecodingError("unsigned_integer".to_string()); "Length too long")]
    #[test_case( &[0x00], 0, Error::DecodingError("unsigned_integer".to_string()); "Length to read is zero")]
    fn read_integer_decoding_err(input: &[u8], length: u8, expected: Error) {
        let err = read_unsigned_integer(&mut std::io::Cursor::new(input), length)
            .expect_err("Testcase should fail here but does not");
        assert_eq!(err.to_string(), expected.to_string())
    }
}
