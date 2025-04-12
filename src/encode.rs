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
    const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_OPENSAFETY_B);

    /// Write the header data into the provided buffer
    ///
    /// The number of written bytes is returned.
    ///
    /// # Panics
    /// This method will panic if `Self::length()` doesn't fit in a single byte.
    /// That would mean the payload is larger than 255 bytes, which is not allowed by the protocol.
    fn write_header_to_buffer(
        &self,
        mut buffer: &mut [u8],
        is_tm_packet: bool,
    ) -> Result<usize, EncodeError> {
        let initial = buffer.remaining_mut();

        buffer.put_u8(self.version());
        buffer.put_u8(Self::length().try_into().unwrap());

        let control = *self.device_id() as u8;
        let control = control | if is_tm_packet { 0 } else { 1 << 7 };
        buffer.put_u8(control);

        buffer.put_u64_le(self.timestamp().0);

        Ok(initial - buffer.remaining_mut())
    }

    fn write_payload_to_buffer(&self, mut buffer: &mut [u8]) -> Result<usize, EncodeError> {
        let initial = buffer.remaining_mut();

        buffer.put_u128_le(self.payload().get());

        Ok(initial - buffer.remaining_mut())
    }

    /// Encode the internal packet into the given buffer
    fn encode(&self, buffer: &mut [u8], is_tm_packet: bool) -> Result<(), EncodeError> {
        let available = buffer.remaining_mut();
        if available < InternalPacket::size() {
            return Err(EncodeError::BufferTooSmall {
                required: InternalPacket::size(),
                available,
            });
        }

        let written = self.write_header_to_buffer(buffer, is_tm_packet)?;

        let written = written + self.write_payload_to_buffer(&mut buffer[written..])?;

        let checksum = Self::CRC.checksum(&buffer[..written]);

        // Write the checksum after what's already written
        (&mut buffer[written..]).put_u16_le(checksum);
        let written = written + 2;

        Ok(())
    }
}

impl TmPacket {
    pub fn encode(&self, buffer: &mut [u8]) -> Result<(), EncodeError> {
        self.0.encode(buffer, true)
    }
}

impl TcPacket {
    pub fn encode(&self, buffer: &mut [u8]) -> Result<(), EncodeError> {
        self.0.encode(buffer, false)
    }
}

impl Packet {
    /// Encode the packet into the given buffer
    pub fn encode(&self, buffer: &mut [u8]) -> Result<(), EncodeError> {
        match self {
            Packet::TmPacket(packet) => packet.encode(buffer),
            Packet::TcPacket(packet) => packet.encode(buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::borrow::BorrowMut;

    use crate::{
        encode::EncodeError, DeviceId, InternalPacket, Packet, Payload, TcPacket, Timestamp,
        TmPacket, VERSION,
    };

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

        let mut buffer = [0u8; InternalPacket::size()];

        packet.encode(buffer.borrow_mut(), true).unwrap();

        assert_eq!(
            buffer,
            &[
                VERSION, 16, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0x4D, 0x4E, 0, 0
            ][..]
        );
    }

    #[test]
    fn internal_packet_encode_tc_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = InternalPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::size()];

        packet.encode(buffer.borrow_mut(), false).unwrap();

        assert_eq!(
            buffer,
            &[
                VERSION, 16, 0b10000000, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0x6F, 0xB1, 0, 0
            ][..]
        );
    }

    #[test]
    fn internal_packet_encode_buffer_too_small() {
        let payload = Payload::new(0xABCDEF);
        let packet = InternalPacket::new(DeviceId::MissingDevice, Timestamp(0), payload);

        let mut buffer = [0u8; InternalPacket::size() - 1];

        let result = packet.encode(buffer.borrow_mut(), true);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, EncodeError::BufferTooSmall { .. }));
        let EncodeError::BufferTooSmall {
            required,
            available,
        } = error;
        assert_eq!(required, InternalPacket::size());
        assert_eq!(available, buffer.len());
    }

    #[test]
    fn tm_packet_encode_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = TmPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; TmPacket::size()];

        packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            buffer,
            &[
                VERSION, 16, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0x4D, 0x4E, 0, 0
            ][..]
        );
    }

    #[test]
    fn tc_packet_encode_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = TcPacket::new(DeviceId::MissingDevice, Timestamp(10), payload);

        let mut buffer = [0u8; TcPacket::size()];

        packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            buffer,
            &[
                VERSION, 16, 0b10000000, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0x6F, 0xB1, 0, 0
            ][..]
        );
    }

    #[test]
    fn packet_encode_tm_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = Packet::TmPacket(TmPacket::new(
            DeviceId::MissingDevice,
            Timestamp(10),
            payload,
        ));

        let mut buffer = [0u8; TmPacket::size()];

        packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            buffer,
            &[
                VERSION, 16, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0x4D, 0x4E, 0, 0
            ][..]
        );
    }

    #[test]
    fn packet_encode_tc_packet_works() {
        let payload = Payload::new(0xABCDEF);
        let packet = Packet::TcPacket(TcPacket::new(
            DeviceId::MissingDevice,
            Timestamp(10),
            payload,
        ));

        let mut buffer = [0u8; TcPacket::size()];

        packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            buffer,
            &[
                VERSION, 16, 0b10000000, 10, 0, 0, 0, 0, 0, 0, 0, 0xEF, 0xCD, 0xAB, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0x6F, 0xB1, 0, 0
            ][..]
        );
    }
}
