use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::{Read, Seek, SeekFrom},
    rc::Rc,
};

use crate::{
    encoding::{self, ber::read_ber},
    klv::Klv,
    universal_set::UNIVERSAL_KEY_LENGTH,
};

// This is just used to make the indexing into the BTreeMap more understandable
type TagNumber = u128;

/// Set of data that must be found in reference to Universal Key
pub struct LocalSet<T>
where
    T: Read + Seek,
{
    /// Locations in the file for each tag that can be parsed.
    data: BTreeMap<TagNumber, Klv<T>>,
}

impl<T> LocalSet<T>
where
    T: Read + Seek,
{
    pub fn new(universal_key_pos: u64, buf: Rc<RefCell<T>>) -> Result<Self, encoding::Error> {
        // Stores all of the tags
        let mut bmap = BTreeMap::new();

        // Location of the first byte that denotes how long the value for the
        // KLV triplet is.
        let length_pos = universal_key_pos + UNIVERSAL_KEY_LENGTH as u64;

        let mut buf_ref = buf.borrow_mut();

        // Move the file pointer to the start of the length
        buf_ref.seek(SeekFrom::Start(length_pos));

        // Length of the value portion of this KLV triplet.
        let value_length: u64 = read_ber(&mut *buf_ref)?
            .try_into()
            .expect("Seek trait only supports u64 values");

        // The value always starts immediately after the length
        let value_start_pos = buf_ref.stream_position().unwrap();
        let final_value_position = value_start_pos + value_length;

        drop(buf_ref);

        while buf.borrow_mut().stream_position().unwrap() != final_value_position + 1 {
            let klv = Klv::new(buf.clone())?;
            bmap.insert(*klv.tag(), klv);
        }

        Ok(Self { data: bmap })
    }
}
