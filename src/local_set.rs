use crate::format::{KlvFormat, SoftwareFormat};

pub enum SpecialValue<T> {
    OutOfRange(T),
}
pub enum ValueLength {
    /// Max Length - specifies the recommended maximum length. With some items
    /// the underlying standard or data structure does not have a limit. If the
    /// Max Length is not determinable it will have a value of "Not Limited."
    /// Network guards may use this value as a check to prevent data leaks.
    Max(u128),

    /// Required Length - specifies a required length if one exists. With a
    /// required length the value portion of the Tag-Length-Value is not to
    /// exceed the number of required length bytes nor the value be less than
    /// the required length. See requirement below.
    Required(u128),

    /// Length - specifies the nominal length to use. If Required Length has a
    /// value other than "N/A" then the length will equal the Required Length. A
    /// length of "Variable" means the length is determined at run-time for the
    /// Tag-Length-Value item.
    Length(u128),
}

pub trait KlvTag<K: KlvFormat> {
    /// Min - specifies the minimum value allowed for the value. When
    /// mapping values the Min(KLV) can be very different than the
    /// Min(Software).
    fn min(&self) -> Option<K>;

    /// Max - specifies the maximum value allowed for the value. When
    /// mapping values the Max(KLV) can be very different than the
    /// Max(Software).
    fn max(&self) -> Option<K>;

    /// Offset (KLV) - specifies the offset used when mapping between software
    /// and KLV formats.
    fn offset(&self) -> Option<K>;

    /// Number of bytes used to store the value for this tag.
    fn length(&self) -> ValueLength;
}

pub trait SoftwareTag<S: SoftwareFormat> {
    /// Min (Software) - specifies the minimum value allowed for the value
    fn min(&self) -> Option<S>;

    /// Max (Software) - specifies the maximum value allowed for the value
    fn max(&self) -> Option<S>;

    /// Special Values - specifies signaling values for numeric values, such as
    /// "Out of Range" or "N/A (Off-Earth)," if they exist for the item. A
    /// Special Value listed as "None" indicates there are no special values,
    /// currently, for the item. A Special Value listed as "N/A" indicates
    /// special values do not apply to the item because it is not a numeric
    /// value (e.g., a string or set are not numeric items).
    fn special_values(&self) -> Vec<SpecialValue<S>>;
}

pub trait Tag<K: KlvFormat, S: SoftwareFormat>: From<u128> + Into<u128> {
    /// A brief description of the item's meaning
    fn description(&self) -> Option<&str>;

    /// Format (Software) - the data format used within a software application
    ///
    fn required(&self) -> bool;

    /// The units used for measured items. "None" indicates the item is not a
    /// measured quantity.
    fn unit(&self) -> Option<&str>;

    /// A Yes or No indication if the item is allowed in a Standard Deviation
    /// Cross Correlation (SDCC) Pack. Yes, indicates the item is allowed in the
    /// SDDC Pack.
    ///
    /// TODO: Figure out what the SDCC Pack is.
    fn allowed_in_sdcc(&self) -> bool;

    /// Defines the method (i.e., an equation) of converting from a Software
    /// Value to its KLV Value.
    fn to_klv_value(&self) -> fn(S) -> K;

    /// Defines the method (i.e., an equation) of converting from a KLV Value to
    /// its Software Value. The KLV Value bit pattern in each equation is
    /// interpretable in diverse ways.
    fn to_software_value(&self) -> fn(K) -> S;
}

pub struct TagValue<T: Tag> {
    a: T,
}

pub struct LocalSet<T: Tag> {
    data: TagValue<T>,
}
