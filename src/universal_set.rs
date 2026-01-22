use byteorder::ReadBytesExt;
use std::{
    cell::RefCell,
    io::{Read, Seek},
    ops::Deref,
    rc::Rc,
};

use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::{encoding, klv::Klv, local_set::LocalSet};

/// Length of a Universal Key is always 16 bytes.
pub const UNIVERSAL_KEY_LENGTH: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct UniversalKey([u8; UNIVERSAL_KEY_LENGTH]);
impl UniversalKey {
    pub fn new(key: [u8; UNIVERSAL_KEY_LENGTH]) -> Self {
        Self(key)
    }
}

impl Deref for UniversalKey {
    type Target = [u8; UNIVERSAL_KEY_LENGTH];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Set of data that can be found by searching for the Universal Key in the
/// file.
pub struct UniversalSet<'a, T>
where
    T: Read + Seek,
{
    /// Key used to find the beginning of the `LocalSet`.
    key: &'a UniversalKey,

    /// Locations in the file for each tag that can be parsed.
    data: LocalSet<T>,
}

impl<'a, T> UniversalSet<'a, T>
where
    T: Read + Seek,
{
    pub fn new(
        key: &'a UniversalKey,
        buf: Rc<RefCell<T>>,
        starting_location: u64,
    ) -> Result<Self, encoding::Error> {
        Ok(Self {
            key,
            data: LocalSet::read(starting_location, buf)?,
        })
    }

    pub fn find_all(
        key: &'a UniversalKey,
        buf: Rc<RefCell<T>>,
    ) -> Result<Vec<UniversalSet<'a, T>>, encoding::Error> {
        let locations = Self::start_locations(key, &mut *buf.borrow_mut())?;
        locations
            .iter()
            .map(|start| UniversalSet::new(key, buf.clone(), *start))
            .collect::<Result<Vec<UniversalSet<'a, T>>, encoding::Error>>()
    }

    /// Return the offsets to the first byte of the Universal Key everywhere the
    /// Universal Key was found in the buffer.
    pub fn start_locations(
        key: &'a UniversalKey,
        buf: &mut T,
    ) -> Result<Vec<u64>, encoding::Error> {
        let mut locations = Vec::new();

        // The initial contents of the search buffer should be the start of the
        // file.
        let mut buffer_contents = [0; UNIVERSAL_KEY_LENGTH];
        if buf.read_exact(&mut buffer_contents).is_ok() {
            let mut search_buffer =
                ConstGenericRingBuffer::<u8, UNIVERSAL_KEY_LENGTH>::from(buffer_contents);

            loop {
                if itertools::equal(&search_buffer, &key.0) {
                    // Matches will only happen after the last byte of the
                    // Universal Key has been read so we always need to subtract
                    // the length of the key from the current position to get
                    // the starting position.
                    let current_pos = buf.stream_position().unwrap();
                    let start_pos = match current_pos.checked_sub(UNIVERSAL_KEY_LENGTH as u64) {
                        Some(pos) => pos,
                        None => panic!(
                            "Starting position of Key with length [{UNIVERSAL_KEY_LENGTH}] ending at index [{current_pos}] results in a negative offset in the buffer"
                        ),
                    };
                    locations.push(start_pos);

                    // Get how far to jump at the very least to get to the next
                    // KLV value.
                    let value_length = Klv::read_length(buf)?;
                    buf.seek_relative(value_length.try_into().unwrap()).unwrap();
                }

                match buf.read_u8() {
                    Ok(val) => {
                        search_buffer.enqueue(val);
                    }
                    Err(_) => break,
                }
            }
        };

        Ok(locations)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use itertools::{Itertools, chain};
    use test_case::test_case;

    const TEST_UNIVERSAL_KEY: [u8; UNIVERSAL_KEY_LENGTH] = [
        0x06, 0x0E, 0x2B, 0x34, 0x02, 0x0B, 0x01, 0x01, 0x0E, 0x01, 0x03, 0x01, 0x01, 0x00, 0x00,
        0x00,
    ];

    #[test_case(&chain!(TEST_UNIVERSAL_KEY, [0x01, 0x00]).collect_vec(), &[0]; "One at beginning")]
    #[test_case(&chain!([0x06], TEST_UNIVERSAL_KEY, [0x01, 0x00]).collect_vec(), &[1]; "One at offset")]
    #[test_case(&chain!([0x06], TEST_UNIVERSAL_KEY, [0x01, 0x06], TEST_UNIVERSAL_KEY, [0x01, 0x00]).collect_vec(), &[1, 19]; "Two at offset")]
    fn test_start_locations(buf: &[u8], expected: &[u64]) {
        let ukey = UniversalKey::new(TEST_UNIVERSAL_KEY);
        assert_eq!(
            UniversalSet::start_locations(&ukey, &mut Cursor::new(buf)).unwrap(),
            *expected
        )
    }
}
