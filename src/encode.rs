use bytes::BufMut;

use crate::{InternalPacket, Packet, TcPacket, TmPacket};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EncodeError {
    BufferTooSmall { required: usize, available: usize },
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            EncodeError::BufferTooSmall {
                required,
                available,
            } => write!(
                f,
                "Buffer too small: required {} bytes, but only {} available",
                required, available
            ),
        }
    }
}

impl core::error::Error for EncodeError {}

impl InternalPacket {
    /// Encode the internal packet into the given buffer
    fn encode<T: BufMut>(&self, mut buffer: T, is_tm_packet: bool) -> Result<(), EncodeError> {
        if buffer.remaining_mut() < InternalPacket::size() as usize {
            return Err(EncodeError::BufferTooSmall {
                required: InternalPacket::size() as usize,
                available: buffer.remaining_mut(),
            });
        }

        buffer.put_u8(self.version());
        buffer.put_u8(Self::length());
        let control = *self.device_id() as u8;
        let control = control | if is_tm_packet { 0 } else { 1 << 7 };
        buffer.put_u8(control);
        buffer.put_u64_le(self.timestamp().0);
        buffer.put_u128_le(self.payload().get());

        Ok(())
    }
}

impl TmPacket {
    pub fn encode<T: BufMut>(&self, buffer: T) -> Result<(), EncodeError> {
        self.0.encode(buffer, true)
    }
}

impl TcPacket {
    pub fn encode<T: BufMut>(&self, buffer: T) -> Result<(), EncodeError> {
        self.0.encode(buffer, false)
    }
}

impl Packet {
    /// Encode the packet into the given buffer
    pub fn encode<T: BufMut>(&self, buffer: T) -> Result<(), EncodeError> {
        match self {
            Packet::TmPacket(packet) => packet.encode(buffer),
            Packet::TcPacket(packet) => packet.encode(buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{encode::EncodeError, DeviceId, InternalPacket, Payload, Timestamp, VERSION};

    #[test]
    fn encode_error_display() {
        let error = EncodeError::BufferTooSmall {
            required: 27,
            available: 26,
        };

        assert_eq!(
            error.to_string(),
            "Buffer too small: required 27 bytes, but only 26 available"
        );
    }

    #[test]
    fn internal_packet_encode_trivial_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = InternalPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; 15 + 16];

        packet.encode(&mut buffer[..], true).unwrap();

        assert_eq!(
            buffer,
            [
                VERSION, 16, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn internal_packet_encode_tc_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = InternalPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; 15 + 16];

        packet.encode(&mut buffer[..], false).unwrap();

        assert_eq!(
            buffer,
            [
                VERSION, 16, 0b10000000, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn internal_packet_encode_buffer_too_small() {
        let payload = Payload::new(0xABCDEF);
        let packet = InternalPacket::new(DeviceId::MissingDevice, Timestamp(0), payload);

        let mut buffer = [0u8; 11];

        let result = packet.encode(&mut buffer[..], true);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            EncodeError::BufferTooSmall {
                required: 31,
                available: 11,
            }
        ));
    }

    #[test]
    fn tm_packet_encode_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = crate::TmPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; 15 + 16];

        packet.encode(&mut buffer[..]).unwrap();

        assert_eq!(
            buffer,
            [
                VERSION, 16, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn tc_packet_encode_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = crate::TcPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; 15 + 16];

        packet.encode(&mut buffer[..]).unwrap();

        assert_eq!(
            buffer,
            [
                VERSION, 16, 0b10000000, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn packet_encode_tm_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = crate::Packet::TmPacket(crate::TmPacket::new(
            DeviceId::MissingDevice,
            Timestamp(10),
            payload,
        ));

        let mut buffer = [0u8; 15 + 16];

        packet.encode(&mut buffer[..]).unwrap();

        assert_eq!(
            buffer,
            [
                VERSION, 16, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn packet_encode_tc_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = crate::Packet::TcPacket(crate::TcPacket::new(
            DeviceId::MissingDevice,
            Timestamp(10),
            payload,
        ));

        let mut buffer = [0u8; 15 + 16];

        packet.encode(&mut buffer[..]).unwrap();

        assert_eq!(
            buffer,
            [
                VERSION, 16, 0b10000000, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
