use core::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PayloadError {
    PayloadTooLong(usize),
}

impl Display for PayloadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PayloadError::PayloadTooLong(length) => {
                write!(f, "payload too long: {} bytes", length)
            }
        }
    }
}

impl core::error::Error for PayloadError {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Payload {
    #[cfg_attr(feature = "serde", serde(with = "serde_with::As::<serde_with::Bytes>"))]
    data: [u8; 255],
    length: usize,
}

impl Payload {
    pub const MAX_SIZE: usize = 255;

    pub fn new() -> Self {
        Self {
            data: [0; 255],
            length: 0,
        }
    }

    pub fn from_raw_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self, PayloadError> {
        let bytes = bytes.as_ref();
        if bytes.len() > Self::MAX_SIZE {
            return Err(PayloadError::PayloadTooLong(bytes.len()));
        }
        let mut payload = Self::new();
        payload.data[..bytes.len()].copy_from_slice(bytes);
        payload.length = bytes.len();
        Ok(payload)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.length]
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<&[u8]> for Payload {
    type Error = PayloadError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_raw_bytes(value)
    }
}

impl AsRef<[u8]> for Payload {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
