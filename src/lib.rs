#![cfg_attr(not(test), no_std)]

//! This crate implements the [OrbiPacket](https://github.com/orbisat-oeiras/orbipacket) protocol,
//! developed for communication with CanSat devices by the OrbiSat Oeiras team.
//!
//! This crate is `no_std` compatible, and can be used in embedded systems. It also doesn't perform any
//! heap allocations.
//!
//! # Basics
//! Packets come in two flavours, each represented by a struct:
//! - [TmPacket]: telemetry packet
//! - [TcPacket]: telecommand packet
//!
//! It is also possible to refer to a general packet using the [Packet] enum, which has variants for
//! both packet types.
//!
//! # Packet structure
//! The packet structs closely follow the protocol's specification, which provides a full reference.
//! A brief summary of the structs' fields is given below:
//! - `version`: indicates the version of the protocol the packet adheres to
//! - `payload_length`: length of the payload, in bytes
//! - `device_id`: see [DeviceId]
//! - `timestamp`: see [Timestamp]
//! - `payload`: application specific data
//!
//! # Encoding
//! Packets can be encoded into a buffer using any of [TmPacket::encode], [TcPacket::encode] or
//! [Packet::encode]. All these methods accept a mutable byte slice to which they write the encoded
//! packet, returning a slice into the buffer guaranteed to contain exactly the packet's bytes.
//!
//! ```rust
//! use orbipacket::{TmPacket, DeviceId, Timestamp, Payload};
//!
//! let packet = TmPacket::new(
//!     DeviceId::System,
//!     Timestamp::new(1234),
//!     Payload::from_bytes(b"hello world")?,
//! );
//! let mut buffer = [1u8; 500];
//!
//! let encoded = packet.encode(&mut buffer)?;
//!
//! assert_eq!(encoded, &[6, 0x01, 11, 4, 0xD2, 0x04, 1, 1, 1, 1, 1, 14, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', 223, 75, 0][..]);
//! assert_eq!(encoded.len(), packet.encoded_size());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//! By dropping the returned slice, the same buffer can be used to encode multiple packets sequentially
//! or inside a loop, allowing for efficient memory usage.
//!
//! ```rust
//! use orbipacket::{TmPacket, DeviceId, Timestamp, Payload};
//!
//! let mut buffer = [0u8; 500];
//!
//! for i in 1..10u8 {
//!     let packet = TmPacket::new(
//!         DeviceId::System,
//!         Timestamp::new(1234),
//!         Payload::from_bytes([i])?,
//!     );
//!
//!     let encoded = packet.encode(&mut buffer)?;
//!
//!     assert!(matches!(encoded, [6, 0x01, 1, 4, 210, 4, 1, 1, 1, 1, 1, 4, i, _, _, 0]));
//!     assert_eq!(encoded.len(), packet.encoded_size());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Buffer size
//! Currently, encoding a packet requires a buffer approximately twice the size of the actual encoded packet.
//! This is necessary because COBS encoding must be done buffer-to-buffer. Thus, the first half of the provided
//! buffer is used to write the packet fields (as a sort of intermediate value), and the second half is then
//! used to write the COBS-encoded packet and returned. This leads to sub-optimal memory usage, which is a
//! compromise made to avoid the use of allocations. The provided constants [TmPacket::MAX_ENCODE_BUFFER_SIZE]
//! and [TmPacket::MAX_ENCODE_BUFFER_SIZE] can be used to allocate buffers large enough to encode any packet.
//! If the buffers are dynamically allocated, then the methods [TmPacket::encode_buffer_size] and
//! [TcPacket::encode_buffer_size] can be used instead to obtain a buffer large enough to encode a specific
//! packet.
//!
//! # Decoding
//! TODO: Decoding isn't implemented yet.

static VERSION: u8 = 0x01;

pub mod payload;
pub use payload::Payload;

use core::fmt::Display;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u8)]
pub enum DeviceId {
    System = 1,
    PressureSensor = 2,
    TemperatureSensor = 3,
    HumiditySensor = 4,
    Gps = 5,
    Accelerometer = 6,
    Unknown = 31,
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceId::System => write!(f, "System Device (ID {})", *self as u8),
            DeviceId::PressureSensor => write!(f, "Pressure Sensor Device (ID {})", *self as u8),
            DeviceId::TemperatureSensor => {
                write!(f, "Temperature Sensor Device (ID {})", *self as u8)
            }
            DeviceId::HumiditySensor => write!(f, "Humidity Sensor Device (ID {})", *self as u8),
            DeviceId::Gps => write!(f, "GPS Device (ID {})", *self as u8),
            DeviceId::Accelerometer => write!(f, "Accelerometer Device (ID {})", *self as u8),
            DeviceId::Unknown => write!(f, "Unknown Device (ID {})", *self as u8),
        }
    }
}

/// Time in nanoseconds since the Unix epoch
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timestamp(u64);

impl Display for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ns", self.0)
    }
}

impl Timestamp {
    pub fn new(timestamp: u64) -> Self {
        Timestamp(timestamp)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

/// A packet containing metadata and a payload
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct InternalPacket {
    version: u8,
    device_id: DeviceId,
    timestamp: Timestamp,
    payload: Payload,
}

impl InternalPacket {
    /// Create a new packet
    fn new(device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        InternalPacket {
            version: VERSION,
            device_id,
            timestamp,
            payload,
        }
    }
}

/// # Packet field getters
impl InternalPacket {
    fn version(&self) -> u8 {
        self.version
    }

    fn device_id(&self) -> &DeviceId {
        &self.device_id
    }

    fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    fn payload(&self) -> &Payload {
        &self.payload
    }
}

/// # Packet size
impl InternalPacket {
    /// Number of bytes introduced by packet metadata
    ///
    /// Corresponds to:
    /// - 1 byte for the version
    /// - 1 byte for the length
    /// - 1 byte for the device ID and packet kind
    /// - 8 bytes for the timestamp
    /// - 2 bytes for the CRC
    const OVERHEAD: usize = 1 + 1 + 1 + 8 + 2;

    const MAX_SIZE: usize = Self::OVERHEAD + Payload::MAX_SIZE;

    /// Maximum size of an encoded packet, in bytes
    const MAX_ENCODED_SIZE: usize = cobs::max_encoding_length(Self::MAX_SIZE) + 1;

    fn size(&self) -> usize {
        Self::OVERHEAD + self.payload.length()
    }

    fn encoded_size(&self) -> usize {
        cobs::max_encoding_length(self.size()) + 1
    }
}

/// A telemetry packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TmPacket(InternalPacket);

impl TmPacket {
    /// Create a new telemetry packet
    pub fn new(device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        TmPacket(InternalPacket::new(device_id, timestamp, payload))
    }
}

/// # Packet field getters
impl TmPacket {
    pub fn device_id(&self) -> &DeviceId {
        self.0.device_id()
    }

    pub fn payload(&self) -> &Payload {
        self.0.payload()
    }

    pub fn timestamp(&self) -> &Timestamp {
        self.0.timestamp()
    }

    pub fn version(&self) -> u8 {
        self.0.version()
    }
}

/// # Packet size
impl TmPacket {
    /// Number of bytes introduced by packet metadata
    ///
    /// Corresponds to:
    /// - 1 byte for the version
    /// - 1 byte for the length
    /// - 1 byte for the device ID and packet kind
    /// - 8 bytes for the timestamp
    /// - 2 bytes for the CRC
    pub const OVERHEAD: usize = InternalPacket::OVERHEAD;

    pub const MAX_SIZE: usize = InternalPacket::MAX_SIZE;

    /// Maximum size of an encoded packet, in bytes
    pub const MAX_ENCODED_SIZE: usize = InternalPacket::MAX_ENCODED_SIZE;

    pub fn size(&self) -> usize {
        self.0.size()
    }

    pub fn encoded_size(&self) -> usize {
        self.0.encoded_size()
    }
}

impl Display for TmPacket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Telemetry packet from {} with timestamp {}",
            self.device_id(),
            self.timestamp()
        )
    }
}

/// A telecommand packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TcPacket(InternalPacket);

impl TcPacket {
    /// Create a new telecommand packet
    pub fn new(device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        TcPacket(InternalPacket::new(device_id, timestamp, payload))
    }
}

/// # Packet field getters
impl TcPacket {
    pub fn device_id(&self) -> &DeviceId {
        self.0.device_id()
    }

    pub fn payload(&self) -> &Payload {
        self.0.payload()
    }

    pub fn timestamp(&self) -> &Timestamp {
        self.0.timestamp()
    }

    pub fn version(&self) -> u8 {
        self.0.version()
    }
}

/// # Packet size
impl TcPacket {
    /// Number of bytes introduced by packet metadata
    ///
    /// Corresponds to:
    /// - 1 byte for the version
    /// - 1 byte for the length
    /// - 1 byte for the device ID and packet kind
    /// - 8 bytes for the timestamp
    /// - 2 bytes for the CRC
    pub const OVERHEAD: usize = InternalPacket::OVERHEAD;

    pub const MAX_SIZE: usize = InternalPacket::MAX_SIZE;

    /// Maximum size of an encoded packet, in bytes
    pub const MAX_ENCODED_SIZE: usize = InternalPacket::MAX_ENCODED_SIZE;

    pub fn size(&self) -> usize {
        self.0.size()
    }

    pub fn encoded_size(&self) -> usize {
        self.0.encoded_size()
    }
}

impl Display for TcPacket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Telecommand packet to {} with timestamp {}",
            self.device_id(),
            self.timestamp()
        )
    }
}

/// Either a telemetry packet or a telecommand packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Packet {
    TmPacket(TmPacket),
    TcPacket(TcPacket),
}

impl Packet {
    pub fn is_tm_packet(&self) -> bool {
        matches!(self, Packet::TmPacket(_))
    }

    pub fn is_tc_packet(&self) -> bool {
        matches!(self, Packet::TcPacket(_))
    }
}

pub mod encode;

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(byte: u8) -> Payload {
        Payload::from_raw_bytes([byte]).unwrap()
    }

    #[test]
    fn timestamp_getters_return_values_from_constructor() {
        let timestamp = Timestamp::new(1234);
        assert_eq!(timestamp.get(), 1234);
    }

    #[test]
    fn tm_packet_getters_return_values_from_constructor() {
        let payload = payload(3u8);
        let tm_packet = TmPacket::new(DeviceId::System, Timestamp(0), payload);
        assert_eq!(tm_packet.version(), VERSION);
        assert_eq!(tm_packet.device_id(), &DeviceId::System);
        assert_eq!(tm_packet.timestamp().0, 0);
        assert_eq!(*tm_packet.payload(), payload);
    }

    #[test]
    fn tm_packet_overhead_returns_correct() {
        assert_eq!(TmPacket::OVERHEAD, 13);
    }

    #[test]
    fn tm_packet_size_returns_size_of_packet() {
        assert_eq!(TmPacket::MAX_ENCODED_SIZE, 15 + 256);
    }

    #[test]
    fn tc_packet_getters_return_values_from_constructor() {
        let payload = payload(3u8);
        let tc_packet = TcPacket::new(DeviceId::System, Timestamp(0), payload);
        assert_eq!(tc_packet.version(), VERSION);
        assert_eq!(tc_packet.device_id(), &DeviceId::System);
        assert_eq!(tc_packet.timestamp().0, 0);
        assert_eq!(*tc_packet.payload(), payload);
    }

    #[test]
    fn tc_packet_overhead_returns_correct() {
        assert_eq!(TcPacket::OVERHEAD, 13);
    }

    #[test]
    fn tc_packet_size_returns_size_of_packet() {
        assert_eq!(TcPacket::MAX_ENCODED_SIZE, 15 + 256);
    }

    #[test]
    fn packet_is_tm_packet_returns_true_for_tm_packet() {
        let payload = payload(3u8);
        let tm_packet = TmPacket::new(DeviceId::System, Timestamp(0), payload);
        let packet = Packet::TmPacket(tm_packet);
        assert!(packet.is_tm_packet());
    }

    #[test]
    fn packet_is_tm_packet_returns_false_for_tc_packet() {
        let payload = payload(3u8);
        let tc_packet = TcPacket::new(DeviceId::System, Timestamp(0), payload);
        let packet = Packet::TcPacket(tc_packet);
        assert!(!packet.is_tm_packet());
    }

    #[test]
    fn packet_is_tc_packet_returns_true_for_tc_packet() {
        let payload = payload(3u8);
        let tc_packet = TcPacket::new(DeviceId::System, Timestamp(0), payload);
        let packet = Packet::TcPacket(tc_packet);
        assert!(packet.is_tc_packet());
    }

    #[test]
    fn packet_is_tc_packet_returns_false_for_tm_packet() {
        let payload = payload(3u8);
        let tm_packet = TmPacket::new(DeviceId::System, Timestamp(0), payload);
        let packet = Packet::TmPacket(tm_packet);
        assert!(!packet.is_tc_packet());
    }
}
