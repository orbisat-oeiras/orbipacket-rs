use bytes::BufMut;

use crate::{InternalPacket, Packet, Payload, TcPacket, TmPacket};

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
                "buffer too small: required {} bytes, but only {} available",
                required, available
            ),
        }
    }
}

impl core::error::Error for EncodeError {}

impl InternalPacket {
    const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_OPENSAFETY_B);

    /// Maximum size of the buffer needed to encode the packet
    ///
    /// The buffer passed to the `encode` method must be at least this size.
    // For encoding, we first write the header, payload and CRC to the buffer (overhead + payload size bytes).
    // Then, we use the remainder of the buffer as the COBS output buffer.
    const MAX_ENCODE_BUFFER_SIZE: usize =
        Self::OVERHEAD + Payload::MAX_SIZE + Self::MAX_ENCODED_SIZE;

    fn encode_buffer_size(&self) -> usize {
        Self::OVERHEAD + self.payload.length() + self.encoded_size()
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
        if available < self.encode_buffer_size() {
            return Err(EncodeError::BufferTooSmall {
                required: self.encode_buffer_size(),
                available,
            });
        }

        let written =
            self.write_header_to_buffer(buffer, is_tm_packet, self.payload.length() as u8);

        let written =
            written + self.write_payload_to_buffer(&mut buffer[written..], self.payload.as_bytes());

        let checksum = Self::CRC.checksum(&buffer[..written]);

        // Write the checksum after what's already written
        (&mut buffer[written..]).put_u16_le(checksum);
        let written = written + 2;

        let (buffer_unencoded, cobs_buffer) = buffer.split_at_mut(written);
        let encoded = cobs::encode(buffer_unencoded, cobs_buffer);
        buffer[written + encoded] = 0;

        Ok(&buffer[written..(written + encoded + 1)])
    }
}

impl TmPacket {
    /// Size of the buffer needed to encode the packet
    ///
    /// The buffer passed to the `encode` method must be at least this size.
    pub const MAX_ENCODE_BUFFER_SIZE: usize = InternalPacket::MAX_ENCODE_BUFFER_SIZE;

    pub fn encode_buffer_size(&self) -> usize {
        self.0.encode_buffer_size()
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
    pub const MAX_ENCODE_BUFFER_SIZE: usize = InternalPacket::MAX_ENCODE_BUFFER_SIZE;

    pub fn encode_buffer_size(&self) -> usize {
        self.0.encode_buffer_size()
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
    pub const MAX_ENCODE_BUFFER_SIZE: usize = InternalPacket::MAX_ENCODE_BUFFER_SIZE;

    pub fn encode_buffer_size(&self) -> usize {
        match self {
            Packet::TmPacket(tm_packet) => tm_packet.encode_buffer_size(),
            Packet::TcPacket(tc_packet) => tc_packet.encode_buffer_size(),
        }
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
            "buffer too small: required 27 bytes, but only 26 available"
        );
    }

    #[test]
    fn internal_packet_encode_tm_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = InternalPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::MAX_ENCODE_BUFFER_SIZE];

        let encoded = packet.encode(buffer.borrow_mut(), true).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 4, 4, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 1, 3, 173, 120, 0][..]
        );
    }

    #[test]
    fn internal_packet_encode_tc_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = InternalPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::MAX_ENCODE_BUFFER_SIZE];

        let encoded = packet.encode(buffer.borrow_mut(), false).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 4, 132, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 1, 3, 118, 176, 0][..]
        );
    }

    #[test]
    fn internal_packet_encode_buffer_too_small() {
        let payload = payload(0xABCDEFu32);
        let packet = InternalPacket::new(DeviceId::System, Timestamp(0), payload);

        let mut buffer = [0u8; 5];

        let result = packet.encode(buffer.borrow_mut(), true);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, EncodeError::BufferTooSmall { .. }));
        let EncodeError::BufferTooSmall {
            required,
            available,
        } = error;
        assert_eq!(required, packet.encode_buffer_size());
        assert_eq!(available, buffer.len());
    }

    #[test]
    fn tm_packet_encode_works() {
        let payload = payload(0xABCDEFu32);
        let packet = TmPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::MAX_ENCODE_BUFFER_SIZE];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 4, 4, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 1, 3, 173, 120, 0][..]
        );
    }

    #[test]
    fn tc_packet_encode_works() {
        let payload = payload(0xABCDEFu32);
        let packet = TcPacket::new(DeviceId::System, Timestamp(10), payload);

        let mut buffer = [0u8; InternalPacket::MAX_ENCODE_BUFFER_SIZE];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 4, 132, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 1, 3, 118, 176, 0][..]
        );
    }

    #[test]
    fn packet_encode_tm_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = Packet::TmPacket(TmPacket::new(DeviceId::System, Timestamp(10), payload));

        let mut buffer = [0u8; InternalPacket::MAX_ENCODE_BUFFER_SIZE];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 4, 4, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 1, 3, 173, 120, 0][..]
        );
    }

    #[test]
    fn packet_encode_tc_packet_works() {
        let payload = payload(0xABCDEFu32);
        let packet = Packet::TcPacket(TcPacket::new(DeviceId::System, Timestamp(10), payload));

        let mut buffer = [0u8; InternalPacket::MAX_ENCODE_BUFFER_SIZE];

        let encoded = packet.encode(buffer.borrow_mut()).unwrap();

        assert_eq!(
            encoded,
            &[5, VERSION, 4, 132, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 1, 3, 118, 176, 0][..]
        );
    }
}
