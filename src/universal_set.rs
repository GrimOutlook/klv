use byteorder::ReadBytesExt;
use std::{
    io::{Read, Seek},
    ops::Deref,
};

use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::local_set::LocalSet;

/// Length of a Universal Key is always 16 bytes.
const UNIVERSAL_KEY_LENGTH: usize = 16;

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
pub struct UniversalSet<'a, 'b, T>
where
    T: Read + Seek,
{
    /// Key used to find the beginning of the `LocalSet`.
    key: &'a UniversalKey,

    /// Locations in the file for each tag that can be parsed.
    data: LocalSet<'b, T>,
}

impl<'a, 'b, T> UniversalSet<'a, 'b, T>
where
    T: Read + Seek,
{
    pub fn new(key: &'a UniversalKey) -> Self {
        Self {
            key,
            data: LocalSet::new(),
        }
    }

    /// Return the offsets to the first byte of the Universal Key everywhere the
    /// Universal Key was found in the buffer.
    fn locations(&self, file: &mut T) -> Vec<u64> {
        let mut locations = Vec::new();

        // The initial contents of the search buffer should be the start of the
        // file.
        let mut buffer_contents = [0; UNIVERSAL_KEY_LENGTH];
        if file.read_exact(&mut buffer_contents).is_ok() {
            let mut search_buffer =
                ConstGenericRingBuffer::<u8, UNIVERSAL_KEY_LENGTH>::from(buffer_contents);

            loop {
                if itertools::equal(&search_buffer, &self.key.0) {
                    // Matches will only happen after the last byte of the
                    // Universal Key has been read so we always need to subtract
                    // the length of the key from the current position to get
                    // the starting position.
                    let current_pos = file.stream_position().unwrap();
                    let start_pos = match current_pos.checked_sub(UNIVERSAL_KEY_LENGTH as u64) {
                        Some(pos) => pos,
                        None => panic!(
                            "Starting position of Key with length [{UNIVERSAL_KEY_LENGTH}] ending at index [{current_pos}] results in a negative offset in the buffer"
                        ),
                    };
                    locations.push(start_pos)
                }

                match file.read_u8() {
                    Ok(val) => {
                        search_buffer.enqueue(val);
                    }
                    Err(_) => break,
                }
            }
        };

        locations
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

    #[test_case(&TEST_UNIVERSAL_KEY, &[0]; "One at beginning")]
    #[test_case(&chain!([0x06], TEST_UNIVERSAL_KEY).collect_vec(), &[1]; "One at offset")]
    fn locations(buf: &[u8], expected: &[u64]) {
        let ukey = UniversalKey::new(TEST_UNIVERSAL_KEY);
        let uset = UniversalSet::<Cursor<&[u8]>>::new(&ukey);
        assert_eq!(uset.locations(&mut Cursor::new(buf)), *expected)
    }
}
