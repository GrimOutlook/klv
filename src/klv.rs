use std::io::{Read, Seek};

pub struct Klv<'a, T>
where
    T: Read + Seek,
{
    /// File that the KLV data is found in
    file: &'a T,

    /// Number of bytes that make up the value for this KLV triplet.
    ///
    /// Because the length is stored in BER format, the max value length that is
    /// "supported" (very large quotes) is 2^(127*8)-1. Because neither the
    /// _Motion Imagery Handbook_ that defines this format nor _ST 0107.5: KLV
    /// Metadata in Motion Imagery_ that stipulates that lengths use BER limit
    /// this amount we use the largest.
    length: u128,

    /// Starting offset in the file for the first byte that makes up the value
    /// for this KLV triplet.
    starting_offset: u128,
}
