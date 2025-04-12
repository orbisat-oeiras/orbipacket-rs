/// Data contained inside a packet
///
/// TODO: Make the payload generic
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Payload(u128);

impl Payload {
    /// Create a new payload
    pub fn new(data: u128) -> Self {
        Payload(data)
    }

    /// Get the data contained in the payload
    pub fn get(&self) -> u128 {
        self.0
    }

    /// Get the length of the payload in bytes
    pub const fn length() -> usize {
        16
    }
}
