#![cfg_attr(not(test), no_std)]

static VERSION: u8 = 0x01;

/// Data contained inside a packet
///
/// TODO: Make the payload generic
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Payload(u128);

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
    fn new(length: u8, device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        InternalPacket {
            version: VERSION,
            length,
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
    pub fn new(length: u8, device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        TmPacket(InternalPacket::new(length, device_id, timestamp, payload))
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
    pub fn new(length: u8, device_id: DeviceId, timestamp: Timestamp, payload: Payload) -> Self {
        TcPacket(InternalPacket::new(length, device_id, timestamp, payload))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tm_packet_getters() {
        let tm_packet = TmPacket::new(0, DeviceId::MissingDevice, Timestamp(0), Payload(0));
        assert_eq!(tm_packet.version(), VERSION);
        assert_eq!(tm_packet.length(), 0);
        assert_eq!(tm_packet.device_id(), &DeviceId::MissingDevice);
        assert_eq!(tm_packet.timestamp().0, 0);
        assert_eq!(tm_packet.payload().0, 0);
    }

    #[test]
    fn test_tc_packet_getters() {
        let tc_packet = TcPacket::new(0, DeviceId::MissingDevice, Timestamp(0), Payload(0));
        assert_eq!(tc_packet.version(), VERSION);
        assert_eq!(tc_packet.length(), 0);
        assert_eq!(tc_packet.device_id(), &DeviceId::MissingDevice);
        assert_eq!(tc_packet.timestamp().0, 0);
        assert_eq!(tc_packet.payload().0, 0);
    }
}
