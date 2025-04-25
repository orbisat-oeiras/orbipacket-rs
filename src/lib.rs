#![cfg_attr(not(test), no_std)]

//! This crate implements the [OrbiPacket](https://github.com/orbisat-oeiras/orbipacket) protocol,
//! developed for communication with CanSat devices by the OrbiSat Oeiras team.
//!
//! This crate is `no_std` compatible, and can be used in embedded systems. It also doesn't perform any
//! heap allocations.
//!
//! # Basics
//! Packets come in to flavours, each represented by a struct:
//! - [TmPacket]: telemetry packet
//! - [TcPacket]: telecommand packet
//!
//! Both packet types are generic over the payload type, which must implement the [Payload] trait.
//!
//! It is also possible to refer to a general packet using the [Packet] enum, which has variants for
//! both packet types.
//!
//! # Packet structure
//! The packet structs closely follow the protocol's specification, which provides a full reference.
//! A brief summary of the structs' fields is given below:
//! - `version`: indicates the version of the protocol the packet adheres to
//! - `payload_length`: length of the payload, in bytes
//1 - `device_id`: see [DeviceId]
//! - `timestamp`: see [Timestamp]
//! - `payload`: the actual data
//!
//! # Encoding
//! Packets provide methods for encoding themselves into a byte slice, which can then be sent over
//! any communication channel.
//!
//! ```rust
//! use orbipacket::{TmPacket, DeviceId, Timestamp, Payload};
//!
//! let packet = TmPacket::new(
//!     DeviceId::MissingDevice,
//!     Timestamp::new(1234),
//!     Payload::from_bytes(b"hello world").unwrap(),
//! );
//! let mut buffer = [0u8; TmPacket::encode_buffer_size()];
//!
//! let encoded = packet.encode(&mut buffer).unwrap();
//!
//! assert_eq!(encoded, &[3, 0x01, 11, 3, 0xD2, 0x04, 1, 1, 1, 1, 1, 14, b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd', 90, 199, 0][..]);
//! ```
//!
//! Note that the `encode` method returns a slice into the provided buffer containing the encoded packet.
//! After that slice is dropped, the buffer can be reused to encode another packet.
//!
//! ## Buffer size
//! Currently, encoding a packet requires a buffer approximately twice the size of the actual encoded packet.
//! This is due to the use of the [corncobs](https://crates.io/crates/corncobs) crate for encoding, which
//! operates buffer-to-buffer. Thus, the first half of the buffer is used to write the packet fields (as a sort
//! of intermediate value), and the second half is then used to write the encoded packet and returned. This
//! leads to sub-optimal memory usage, which is a compromise made to avoid the use of allocations.
//!
//! # Decoding
//! TODO: Decoding isn't implemented yet.
//!
//! # Payload
//! All packets are generic over the payload type, which is bound by the [Payload] trait. This trait provides
//! a method, `to_le_bytes()` to convert the payload into a byte slice, which is used for encoding. It also checks, at compile
//! time, that the payload size is less than 256 bytes, as required by the protocol. However, there's a slight
//! catch: due to the nature of compile time size checks, the assertion is evaluated only when `to_le_bytes()`
//! is called. This means that a payload type larger than 255 bytes won't result in any error whatsoever, unless
//! an attempt is made to encode it. This is sub-optimal, but it is enough to assure the protocol is followed.

static VERSION: u8 = 0x01;

pub mod payload;
pub use payload::Payload;

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u8)]
pub enum DeviceId {
    System = 1,
    PressureSensor = 2,
    TemperatureSensor = 3,
    HumiditySensor = 4,
    GPS = 5,
    Accelerometer = 6,
    Unknown = 31,
}

/// Time in nanoseconds since the Unix epoch
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Timestamp(u64);

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
    const fn overhead() -> usize {
        1 + 1 + 1 + 8 + 2
    }

    /// Length of the payload contained in a packet, in bytes
    const fn payload_length() -> usize {
        Payload::SIZE
    }

    /// Total size of the packet, in bytes
    const fn size() -> usize {
        corncobs::max_encoded_len(Self::overhead() + Self::payload_length())
    }
}

/// A telemetry packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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
    pub const fn overhead() -> usize {
        InternalPacket::overhead()
    }

    /// Length of the payload contained in a packet, in bytes
    pub const fn payload_length() -> usize {
        InternalPacket::payload_length()
    }

    /// Total size of the packet, in bytes
    pub const fn size() -> usize {
        InternalPacket::size()
    }
}

/// A telecommand packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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
    pub const fn overhead() -> usize {
        InternalPacket::overhead()
    }

    /// Length of the payload contained in a packet, in bytes
    pub const fn payload_length() -> usize {
        InternalPacket::payload_length()
    }

    /// Total size of the packet, in bytes
    pub const fn size() -> usize {
        InternalPacket::size()
    }
}

/// Either a telemetry packet or a telecommand packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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
        Payload::from_bytes(&[byte]).unwrap()
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
    fn tm_packet_length_returns_size_of_payload() {
        assert_eq!(TmPacket::payload_length(), Payload::SIZE);
    }

    #[test]
    fn tm_packet_overhead_returns_correct() {
        assert_eq!(TmPacket::overhead(), 13);
    }

    #[test]
    fn tm_packet_size_returns_size_of_packet() {
        assert_eq!(TmPacket::size(), 15 + 256);
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
    fn tc_packet_length_returns_size_of_payload() {
        assert_eq!(TcPacket::payload_length(), Payload::SIZE);
    }

    #[test]
    fn tc_packet_overhead_returns_correct() {
        assert_eq!(TcPacket::overhead(), 13);
    }

    #[test]
    fn tc_packet_size_returns_size_of_packet() {
        assert_eq!(TcPacket::size(), 15 + 256);
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
