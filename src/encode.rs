use bytes::BufMut;

use crate::{InternalPacket, Packet, TcPacket, TmPacket};

/// Error that can occur when encoding a packet
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EncodeError {
    /// The provided buffer is too small to hold the encoded packet
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

    /// Size of the buffer needed to encode the packet
    ///
    /// The buffer passed to the `encode` method must be at least this size.
    pub const fn encode_buffer_size() -> usize {
        // For encoding, we first write the header, payload and CRC to the buffer (overhead + length bytes).
        // Then, we use the remainder of the buffer as the COBS output buffer.
        Self::overhead() + Self::payload_length() + Self::size()
    }

    /// Write the header data into the provided buffer
    ///
    /// The number of written bytes is returned.
    ///
    /// # Panics
    /// This method will panic if `Self::length()` doesn't fit in a single byte.
    /// That would mean the payload is larger than 255 bytes, which is not allowed by the protocol.
    /// Of course, since `Payload` does a compile time check for this, this function should never panic.
    fn write_header_to_buffer(
        &self,
        mut buffer: &mut [u8],
        is_tm_packet: bool,
        payload_length: u8,
    ) -> usize {
        let initial = buffer.remaining_mut();

        buffer.put_u8(self.version());
        buffer.put_u8(payload_length);

        let control = *self.device_id() as u8;
        let control = control << 2 | if is_tm_packet { 0 } else { 1 << 7 };
        buffer.put_u8(control);

        buffer.put_u64_le(self.timestamp().0);

        initial - buffer.remaining_mut()
    }

    /// Write the payload data into the provided buffer
    ///
    /// The number of written bytes is returned.
    fn write_payload_to_buffer(&self, mut buffer: &mut [u8], payload: &[u8]) -> usize {
        let initial = buffer.remaining_mut();

        buffer.put_slice(payload);

        initial - buffer.remaining_mut()
    }

    /// Encode the packet into the given buffer. Returns a slice of the buffer containing the
    /// encoded packet.
    ///
    /// The provided buffer must be at least `Self::encode_buffer_size()` bytes long.
    fn encode<'a>(
        &self,
        buffer: &'a mut [u8],
        is_tm_packet: bool,
    ) -> Result<&'a [u8], EncodeError> {
        let available = buffer.remaining_mut();
        if available < InternalPacket::encode_buffer_size() {
            return Err(EncodeError::BufferTooSmall {
                required: InternalPacket::encode_buffer_size(),
                available,
            });
        }

        let payload = self.payload.as_bytes();

        let written = self.write_header_to_buffer(buffer, is_tm_packet, payload.len() as u8);

        let written = written + self.write_payload_to_buffer(&mut buffer[written..], payload);

        let checksum = Self::CRC.checksum(&buffer[..written]);

        // Write the checksum after what's already written
        (&mut buffer[written..]).put_u16_le(checksum);
        let written = written + 2;

        let (buffer_unencoded, cobs_buffer) = buffer.split_at_mut(written);
        let encoded = corncobs::encode_buf(buffer_unencoded, cobs_buffer);

        Ok(&buffer[written..(written + encoded)])
    }
}

impl TmPacket {
    /// Size of the buffer needed to encode the packet
    ///
    /// The buffer passed to the `encode` method must be at least this size.
    pub const fn encode_buffer_size() -> usize {
        InternalPacket::encode_buffer_size()
    }

    /// Encode the packet into the given buffer. Returns a slice of the buffer containing the
    /// encoded packet.
    ///
    /// The provided buffer must be at least `Self::encode_buffer_size()` bytes long.
    pub fn encode<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        self.0.encode(buffer, true)
    }
}

impl TcPacket {
    /// Size of the buffer needed to encode the packet
    ///
    /// The buffer passed to the `encode` method must be at least this size.
    pub const fn encode_buffer_size() -> usize {
        InternalPacket::encode_buffer_size()
    }

    /// Encode the packet into the given buffer. Returns a slice of the buffer containing the
    /// encoded packet.
    ///
    /// The provided buffer must be at least `Self::encode_buffer_size()` bytes long.
    pub fn encode<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        self.0.encode(buffer, false)
    }
}

impl Packet {
    /// Size of the buffer needed to encode the packet
    ///
    /// The buffer passed to the `encode` method must be at least this size.
    pub const fn encode_buffer_size() -> usize {
        InternalPacket::encode_buffer_size()
    }

    /// Encode the packet into the given buffer. Returns a slice of the buffer containing the
    /// encoded packet.
    ///
    /// The provided buffer must be at least `Self::encode_buffer_size()` bytes long.
    pub fn encode<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
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

    fn payload(data: u32) -> Payload {
        Payload::from_bytes(data.to_le_bytes().as_slice()).unwrap()
    }

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
    fn internal_packet_encode_tm_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = InternalPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::encode_buffer_size()];

        let encoded = packet.encode(buffer.borrow_mut(), true).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 3, 4, 10, 1, 1, 1, 1, 1, 1, 6, 0xEF, 0xCD, 0xAB, 118, 221, 0][..]
        );
    }

    #[test]
    fn internal_packet_encode_tc_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = InternalPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::encode_buffer_size()];

        let encoded = packet.encode(buffer.borrow_mut(), false).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 3, 132, 10, 1, 1, 1, 1, 1, 1, 6, 0xEF, 0xCD, 0xAB, 101, 185, 0][..]
        );
    }

    #[test]
    fn internal_packet_encode_buffer_too_small() {
        let payload = payload(0xABCDEFu32);
        let packet = InternalPacket::new(DeviceId::System, Timestamp(0), payload);

        let mut buffer = [0u8; InternalPacket::encode_buffer_size() - 1];

        let result = packet.encode(buffer.borrow_mut(), true);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, EncodeError::BufferTooSmall { .. }));
        let EncodeError::BufferTooSmall {
            required,
            available,
        } = error;
        assert_eq!(required, InternalPacket::encode_buffer_size());
        assert_eq!(available, buffer.len());
    }

    #[test]
    fn tm_packet_encode_works() {
        let payload = payload(0xABCDEFu32);
        let packet = TmPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::encode_buffer_size()];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 3, 4, 10, 1, 1, 1, 1, 1, 1, 6, 0xEF, 0xCD, 0xAB, 118, 221, 0][..]
        );
    }

    #[test]
    fn tc_packet_encode_works() {
        let payload = payload(0xABCDEFu32);
        let packet = TcPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::encode_buffer_size()];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 3, 132, 10, 1, 1, 1, 1, 1, 1, 6, 0xEF, 0xCD, 0xAB, 101, 185, 0][..]
        );
    }

    #[test]
    fn packet_encode_tm_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = Packet::TmPacket(TmPacket::new(DeviceId::System, Timestamp(10), payload));

        let mut buffer = [0u8; InternalPacket::encode_buffer_size()];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 3, 4, 10, 1, 1, 1, 1, 1, 1, 6, 0xEF, 0xCD, 0xAB, 118, 221, 0][..]
        );
    }

    #[test]
    fn packet_encode_tc_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = Packet::TcPacket(TcPacket::new(DeviceId::System, Timestamp(10), payload));

        let mut buffer = [0u8; InternalPacket::encode_buffer_size()];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 3, 132, 10, 1, 1, 1, 1, 1, 1, 6, 0xEF, 0xCD, 0xAB, 101, 185, 0][..]
        );
    }
}
