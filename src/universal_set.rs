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
#[derive(Debug, getset::Getters)]
#[getset(get = "pub")]
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

    pub fn read_all(
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
                    let current_pos = buf.stream_position().expect(
                        "Failed to current current buffer position when parsing Universal Set",
                    );
                    let start_pos = match current_pos.checked_sub(UNIVERSAL_KEY_LENGTH as u64) {
                        Some(pos) => pos,
                        None => panic!(
                            "Starting position of Key with length [{UNIVERSAL_KEY_LENGTH}] ending at index [{current_pos}] results in a negative offset in the buffer"
                        ),
                    };
                    locations.push(start_pos);

                    // Get how far to jump at the very least to get to the next
                    // Universal Key.
                    let value_length = Klv::read_length(buf)?;
                    buf.seek_relative(
                        value_length
                            .try_into()
                            .expect("Failed to convert u64 to i64 trying to jump over value"),
                    )
                    .expect("Failed to jump over value");
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

    fn multiple_uset_buf() -> Vec<u8> {
        chain!(
            // Byte before the key starts
            [0x06],
            TEST_UNIVERSAL_KEY, // Starts at index 1
            [0x04],
            [0x01, 0x02, 0x03, 0x04],
            // Bytes that should not be included in either Universal Set
            [0xFF, 0xFF, 0x06],
            TEST_UNIVERSAL_KEY,
            [0x07],
            [0x01, 0x01, 0x02],
            [0x02, 0x02, 0x04, 0x08]
        )
        .collect_vec()
    }

    #[test_case(&chain!(TEST_UNIVERSAL_KEY, [0x02, 0x01, 0x01]).collect_vec(), &[0]; "One at beginning")]
    #[test_case(&chain!([0x06], TEST_UNIVERSAL_KEY, [0x02, 0x01, 0x00]).collect_vec(), &[1]; "One at offset")]
    #[test_case(&multiple_uset_buf(), &[1, 25]; "Two at offset")]
    fn test_start_locations(buf: &[u8], expected: &[u64]) {
        let ukey = UniversalKey::new(TEST_UNIVERSAL_KEY);
        assert_eq!(
            UniversalSet::start_locations(&ukey, &mut Cursor::new(buf)).unwrap(),
            *expected
        )
    }

    #[test]
    fn test_read_all() {
        let ukey = UniversalKey::new(TEST_UNIVERSAL_KEY);
        let mut binding = Cursor::new(multiple_uset_buf());
        let sets = UniversalSet::read_all(&ukey, Rc::new(RefCell::new(&mut binding))).unwrap();

        assert_eq!(sets.len(), 2, "Number of universal sets found is incorrect");
        let first_uset = sets.first().unwrap();
        assert_eq!(
            first_uset.data().len(),
            1,
            "Number of KLV triplets in first universal set is incorrect"
        );
        let first_uset_data = first_uset.data();
        let only_klv = first_uset_data.iter().exactly_one().unwrap().1;
        assert_eq!(
            only_klv.tag(),
            1,
            "Parsed tag value for only KLV triplet in first universal set in incorrect"
        );
        assert_eq!(
            only_klv.length(),
            2,
            "Parsed length for only KLV triplet in first universal set in incorrect"
        );
        assert_eq!(
            only_klv.read_value().unwrap(),
            vec![0x03, 0x04],
            "Parsed value for only KLV triplet in first universal set in incorrect"
        );
    }
}
