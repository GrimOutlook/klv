use std::cell::RefCell;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::rc::Rc;

use crate::encoding;
use crate::encoding::ber::read_ber;
use crate::encoding::ber_oid::read_ber_oid;

pub type RawValueData = Vec<u8>;

#[derive(Debug, getset::CopyGetters)]
pub struct Klv<T>
where
    T: Read + Seek,
{
    /// Reference to the buffer that the KLV data is found in
    buf: Rc<RefCell<T>>,

    /// Number that identifies this KLV triplet in a LocalSet.
    #[getset(get_copy = "pub")]
    tag: u128,

    /// Number of bytes that make up the value for this KLV triplet.
    ///
    /// Because the length is stored in BER format, the max value length that is
    /// "supported" (very large quotes) is 2^(127*8)-1. Because neither the
    /// _Motion Imagery Handbook_ that defines this format nor _ST 0107.5: KLV
    /// Metadata in Motion Imagery_ that stipulates that lengths use BER limit
    /// this amount we use the largest uint container that the Seek trait can
    /// handle which is a `u64`.
    #[getset(get_copy = "pub")]
    length: u64,

    /// Starting offset in the file for the first byte that makes up the value
    /// for this KLV triplet.
    value_offset: u64,
}

impl<T> Klv<T>
where
    T: Read + Seek,
{
    /// Reads in a new KLV triplet from the current buffer position, using the
    /// current position as the start of the Tag data.
    ///
    /// # Returns
    ///
    /// - Ok(Klv) - When the tag number and length can successfully be read and
    ///   parsed.
    /// - Err(std::io::Error) - When there was an issue reading the buffer or
    ///   tag/length couldn't be parsed
    ///
    /// # Side Effects
    ///
    /// Moves the current position in the buffer to the byte after the last
    /// byte of the value.
    pub fn new(buf: Rc<RefCell<T>>) -> Result<Self, encoding::Error> {
        let mut buf_ref = buf.borrow_mut();

        let tag = Self::read_tag(&mut *buf_ref)?;
        let length = Self::read_length(&mut *buf_ref)?;
        let starting_offset = buf_ref.stream_position().unwrap();
        // Move the cursor position to the next byte after the value
        buf_ref.seek_relative(length.try_into().unwrap()).unwrap();

        drop(buf_ref);

        Ok(Self {
            buf,
            tag,
            length,
            value_offset: starting_offset,
        })
    }

    /// Reads the tag number from the current buffer location
    ///
    /// Tag numbers are always stored in BER-OID format according to the `ST
    /// 0107.5 KLV Metadata in Motion Imagery` document section `6.3.1`.
    ///
    /// # Returns
    ///
    /// - Ok(u128) - When a valid u128 BER-OID value can be read from the given
    ///   buffer.
    /// - Err(std::io::Error) - When a valid u128 BER-OID value cannot be read
    ///   from the given buffer.
    ///
    /// # Side Effects
    ///
    /// Moves the current position in the buffer to the byte after the last
    /// BER-OID byte.
    ///
    /// # Panics
    ///
    /// - The value parsed from the BER-OID won't fit in a u128.
    pub fn read_tag(buf: &mut T) -> Result<u128, io::Error> {
        read_ber_oid(buf)
    }

    /// Reads the length of the KLV value from the current buffer location
    ///
    /// Value lengthjs are always stored in BER format according to the `ST
    /// 0107.5 KLV Metadata in Motion Imagery` document section `6.3.2`.
    ///
    /// # Returns
    ///
    /// - Ok(u128) - When a valid u128 BER value can be read from the given
    ///   buffer.
    /// - Err(std::io::Error) - When a valid u128 BER value cannot be read from
    ///   the given buffer.
    ///
    /// # Side Effects
    ///
    /// Moves the current position in the buffer to the byte after the last BER
    /// byte.
    ///
    /// # Panics
    ///
    /// - The value parsed from the BER is long-form and won't fit in a u128.
    pub fn read_length(buf: &mut T) -> Result<u64, io::Error> {
        read_ber(buf).map(|val| {
            val.try_into().expect(
                "Seek trait only supports 64 bit integers but Length requiring 128 bit integer was found",
            )
        })
    }

    /// Returns a copy of the bytes making up the value.
    pub fn read_value(&self) -> Result<Vec<u8>, io::Error> {
        let mut buf = self.buf.borrow_mut();
        let current_position = buf.stream_position().unwrap();
        buf.seek(SeekFrom::Start(self.value_offset)).unwrap();

        let mut temp_buf = vec![0; self.length as usize];
        buf.read_exact(&mut temp_buf)?;

        buf.seek(SeekFrom::Start(current_position)).unwrap();

        Ok(temp_buf)
    }
}
