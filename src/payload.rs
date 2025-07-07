use core::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The error type for operations interacting with [`Payload`]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PayloadError {
    /// The provided data is too long to form a valid payload. The length ot the provided data is
    /// returned as the contents of this variant.
    PayloadTooLong(usize),
}

impl Display for PayloadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PayloadError::PayloadTooLong(length) => {
                write!(f, "payload too long: {length} bytes")
            }
        }
    }
}

impl core::error::Error for PayloadError {}

/// The contents of a packet.
///
/// Internally, the payload is stored as a little endian byte sequence, since that's the format
/// used by the protocol.
///
/// # Example
/// ```
/// # use orbipacket::{Payload};
/// let payload = Payload::from_raw_bytes(255u16.to_le_bytes())?;
/// assert_eq!(payload.as_bytes(), [0xFF, 0x00]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Payload {
    #[cfg_attr(feature = "serde", serde(with = "serde_with::As::<serde_with::Bytes>"))]
    data: [u8; 255],
    length: usize,
}

impl Payload {
    /// Maximum size of a valid payload.
    pub const MAX_SIZE: usize = 255;

    /// Create an empty payload.
    ///
    /// # Example
    /// ```
    /// # use orbipacket::Payload;
    /// let payload = Payload::new();
    /// assert_eq!(payload.as_bytes(), []);
    /// ```
    pub fn new() -> Self {
        Self {
            data: [0; 255],
            length: 0,
        }
    }

    /// Create a payload with the given contents.
    ///
    /// # Warning
    /// This method expects bytes in little endian. Failing to uphold this invariant constitutes
    /// a protocol violation, and can lead to incorrect data transmission.
    ///
    /// TODO: add ways to safely create payloads from common data types.
    ///
    /// # Errors
    /// If the provided bytes are larger than the allowed payload size ([`Payload::MAX_SIZE`]), an error
    /// variant will be returned.
    ///
    /// # Examples
    /// ```
    /// # use orbipacket::Payload;
    /// let payload = Payload::from_raw_bytes(255u16.to_le_bytes())?;
    /// assert_eq!(payload.as_bytes(), [0xFF, 0x00]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ```should_panic
    /// # use orbipacket::Payload;
    /// // On the transmitter side
    /// let original_data = 255u16;
    /// // This violates a protocol invariant
    /// let payload = Payload::from_raw_bytes(original_data.to_be_bytes())?;
    ///
    /// // On the receiver side
    /// let data = u16::from_le_bytes(payload.as_bytes().try_into()?);
    /// // This panics because the protocol was misused
    /// assert_eq!(data, original_data);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Returns the byte representation of the payload.
    ///
    /// # Example
    /// ```
    /// # use orbipacket::{Payload};
    /// let payload = Payload::from_raw_bytes([0xAB, 0xCD, 0xEF])?;
    /// assert_eq!(payload.as_bytes(), [0xAB, 0xCD, 0xEF]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.length]
    }

    /// The length of the payload, in bytes.
    ///
    /// # Example
    /// ```
    /// # use orbipacket::Payload;
    /// let data = [0xAB, 0xCD, 0xEF];
    /// let payload = Payload::from_raw_bytes(&data)?;
    /// assert_eq!(payload.length(), data.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
