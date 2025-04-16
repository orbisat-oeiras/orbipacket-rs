use core::fmt::Display;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PayloadError {
    PayloadTooLong { length: usize },
}

impl Display for PayloadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PayloadError::PayloadTooLong { length } => {
                write!(f, "Payload too long: {} bytes", length)
            }
        }
    }
}

impl core::error::Error for PayloadError {}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Payload([u8; 255]);

impl Payload {
    pub const SIZE: usize = 255;

    pub fn new() -> Self {
        Self([0; 255])
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PayloadError> {
        if bytes.len() > 255 {
            return Err(PayloadError::PayloadTooLong {
                length: bytes.len(),
            });
        }
        let mut payload = Self::new();
        payload.0[..bytes.len()].copy_from_slice(bytes);
        Ok(payload)
    }

    pub fn as_bytes(&self) -> &[u8] {
        // Return a slice of the payload array up to the last non-zero byte
        let mut last_non_zero = 0;
        for (i, &byte) in self.0.iter().enumerate() {
            if byte != 0 {
                last_non_zero = i;
            }
        }
        &self.0[..=last_non_zero]
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
        Self::from_bytes(value)
    }
}

impl AsRef<[u8]> for Payload {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
