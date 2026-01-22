/// The data format used within a software application to represent the value of
/// an item.
pub enum KlvFormat {
    Int,
    Int8,
    Int16,
    Int32,
    Uint,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    IMAPB,
    Byte,
    DLP,
    VLP,
    FLP,
    Set,
    UTF8,
}

/// The data format used within a software application to represent the value of
/// an item.
pub enum SoftwareFormat {
    Byte,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    String,
    Record,
    List,
}
