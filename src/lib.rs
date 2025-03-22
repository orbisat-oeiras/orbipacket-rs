#![cfg_attr(not(test), no_std)]

static VERSION: u8 = 0x01;

/// Data contained inside a packet
///
/// TODO: Make the payload generic
pub struct Payload(u128);

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
pub enum DeviceId {
    MissingDevice,
}

/// Time in nanoseconds since the Unix epoch
pub struct Timestamp(u64);

/// A packet containing metadata and a payload
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
