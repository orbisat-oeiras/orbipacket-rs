#![cfg_attr(not(test), no_std)]

static VERSION: u8 = 0x01;

pub mod payload;
use payload::Payload;

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u8)]
pub enum DeviceId {
    MissingDevice,
}

/// Time in nanoseconds since the Unix epoch
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Timestamp(u64);

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

    /// Number of bytes introduced by packet metadata
    /// Corresponds to:
    /// - 1 byte for the version
    /// - 1 byte for the length
    /// - 1 byte for the device ID and packet kind
    /// - 8 bytes for the timestamp
    /// - 2 bytes for the CRC
    const fn overhead() -> usize {
        1 + 1 + 1 + 8 + 2
    }

    const fn length() -> usize {
        Payload::length()
    }

    const fn size() -> usize {
        corncobs::max_encoded_len(Self::overhead() + Self::length())
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

    pub const fn overhead() -> usize {
        InternalPacket::overhead()
    }

    pub const fn length() -> usize {
        InternalPacket::length()
    }

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

    pub const fn overhead() -> usize {
        InternalPacket::overhead()
    }

    pub const fn length() -> usize {
        InternalPacket::length()
    }

    pub const fn size() -> usize {
        InternalPacket::size()
    }
}

/// A packet is either a telemetry packet or a telecommand packet
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

    #[test]
    fn payload_get_returns_value_from_constructor() {
        let payload = Payload::new(0);
        assert_eq!(payload.get(), 0);
    }

    #[test]
    fn payload_length_returns_size_of_u128() {
        assert_eq!(Payload::length(), core::mem::size_of::<u128>());
    }

    #[test]
    fn tm_packet_getters_return_values_from_constructor() {
        let payload = Payload::new(0);
        let tm_packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(0), payload);
        assert_eq!(tm_packet.version(), VERSION);
        assert_eq!(tm_packet.device_id(), &DeviceId::MissingDevice);
        assert_eq!(tm_packet.timestamp().0, 0);
        assert_eq!(tm_packet.payload().get(), 0);
    }

    #[test]
    fn tm_packet_length_returns_size_of_payload() {
        assert_eq!(TmPacket::length(), Payload::length());
    }

    #[test]
    fn tm_packet_overhead_returns_correct() {
        assert_eq!(TmPacket::overhead(), 13);
    }

    #[test]
    fn tm_packet_size_returns_size_of_packet() {
        assert_eq!(TmPacket::size(), 15 + 16);
    }

    #[test]
    fn tc_packet_getters_return_values_from_constructor() {
        let payload = Payload::new(0);
        let tc_packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(0), payload);
        assert_eq!(tc_packet.version(), VERSION);
        assert_eq!(tc_packet.device_id(), &DeviceId::MissingDevice);
        assert_eq!(tc_packet.timestamp().0, 0);
        assert_eq!(tc_packet.payload().get(), 0);
    }

    #[test]
    fn tc_packet_length_returns_size_of_payload() {
        assert_eq!(TcPacket::length(), Payload::length());
    }

    #[test]
    fn tc_packet_overhead_returns_correct() {
        assert_eq!(TcPacket::overhead(), 13);
    }

    #[test]
    fn tc_packet_size_returns_size_of_packet() {
        assert_eq!(TcPacket::size(), 15 + 16);
    }

    #[test]
    fn packet_is_tm_packet_returns_true_for_tm_packet() {
        let tm_packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload::new(0));
        let packet = Packet::TmPacket(tm_packet);
        assert!(packet.is_tm_packet());
    }

    #[test]
    fn packet_is_tm_packet_returns_false_for_tc_packet() {
        let tc_packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload::new(0));
        let packet = Packet::TcPacket(tc_packet);
        assert!(!packet.is_tm_packet());
    }

    #[test]
    fn packet_is_tc_packet_returns_true_for_tc_packet() {
        let tc_packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload::new(0));
        let packet = Packet::TcPacket(tc_packet);
        assert!(packet.is_tc_packet());
    }

    #[test]
    fn packet_is_tc_packet_returns_false_for_tm_packet() {
        let tm_packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload::new(0));
        let packet = Packet::TmPacket(tm_packet);
        assert!(!packet.is_tc_packet());
    }
}
