#![cfg_attr(not(test), no_std)]

static VERSION: u8 = 0x01;

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
    pub fn length(&self) -> u8 {
        core::mem::size_of::<u128>().try_into().unwrap()
    }
}

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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
    length: u8,
    device_id: DeviceId,
    timestamp: Timestamp,
    payload: Payload,
}

impl InternalPacket {
    /// Create a new packet
    fn new(device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        InternalPacket {
            version: VERSION,
            length: payload.length(),
            device_id,
            timestamp,
            payload,
        }
    }

    fn version(&self) -> u8 {
        self.version
    }

    fn length(&self) -> u8 {
        self.length
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

    pub fn length(&self) -> u8 {
        self.0.length()
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

    pub fn length(&self) -> u8 {
        self.0.length()
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
        let payload = Payload::new(0);
        assert_eq!(
            payload.length(),
            core::mem::size_of::<u128>().try_into().unwrap()
        );
    }

    #[test]
    fn tm_packet_getters_return_values_from_constructor() {
        let payload = Payload::new(0);
        let tm_packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(0), payload);
        assert_eq!(tm_packet.version(), VERSION);
        assert_eq!(tm_packet.length(), payload.length());
        assert_eq!(tm_packet.device_id(), &DeviceId::MissingDevice);
        assert_eq!(tm_packet.timestamp().0, 0);
        assert_eq!(tm_packet.payload().0, 0);
    }

    #[test]
    fn tc_packet_getters_return_values_from_constructor() {
        let payload = Payload::new(0);
        let tc_packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(0), payload);
        assert_eq!(tc_packet.version(), VERSION);
        assert_eq!(tc_packet.length(), payload.length());
        assert_eq!(tc_packet.device_id(), &DeviceId::MissingDevice);
        assert_eq!(tc_packet.timestamp().0, 0);
        assert_eq!(tc_packet.payload().0, 0);
    }

    #[test]
    fn packet_is_tm_packet_returns_true_for_tm_packet() {
        let tm_packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload(0));
        let packet = Packet::TmPacket(tm_packet);
        assert!(packet.is_tm_packet());
    }

    #[test]
    fn packet_is_tm_packet_returns_false_for_tc_packet() {
        let tc_packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload(0));
        let packet = Packet::TcPacket(tc_packet);
        assert!(!packet.is_tm_packet());
    }

    #[test]
    fn packet_is_tc_packet_returns_true_for_tc_packet() {
        let tc_packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload(0));
        let packet = Packet::TcPacket(tc_packet);
        assert!(packet.is_tc_packet());
    }

    #[test]
    fn packet_is_tc_packet_returns_false_for_tm_packet() {
        let tm_packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(0), Payload(0));
        let packet = Packet::TmPacket(tm_packet);
        assert!(!packet.is_tc_packet());
    }
}
