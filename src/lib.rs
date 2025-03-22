#![cfg_attr(not(test), no_std)]

/// Data contained inside a packet
///
/// TODO: Make the payload generic
pub struct Payload(u128);

/// The ID of a device onboard the CanSat, as specified by the protocol
///
/// TODO: Autogenerate the enum variants from the protocol mapping
pub enum DeviceId {}

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

/// A telemetry packet
pub struct TmPacket(InternalPacket);

impl TmPacket {
    /// Create a new telemetry packet
    pub fn new(
        version: u8,
        length: u8,
        device_id: DeviceId,
        timestamp: Timestamp,
        payload: Payload,
    ) -> Self {
        TmPacket(InternalPacket {
            version,
            length,
            device_id,
            timestamp,
            payload,
        })
    }
}

/// A telecommand packet
pub struct TcPacket(InternalPacket);

impl TcPacket {
    /// Create a new telecommand packet
    pub fn new(
        version: u8,
        length: u8,
        device_id: DeviceId,
        timestamp: Timestamp,
        payload: Payload,
    ) -> Self {
        TcPacket(InternalPacket {
            version,
            length,
            device_id,
            timestamp,
            payload,
        })
    }
}

/// A packet is either a telemetry packet or a telecommand packet
pub enum Packet {
    TmPacket(TmPacket),
    TcPacket(TcPacket),
}
