use std::{
    collections::BTreeMap,
    io::{Read, Seek},
};

use crate::klv::Klv;

// This is just used to make the indexing into the BTreeMap more understandable
type TagNumber = u128;

/// Set of data that must be found in reference to Universal Key
pub struct LocalSet<'b, T>
where
    T: Read + Seek,
{
    /// Locations in the file for each tag that can be parsed.
    data: BTreeMap<TagNumber, Klv<'b, T>>,
}

impl<'b, T> LocalSet<'b, T>
where
    T: Read + Seek,
{
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
}
