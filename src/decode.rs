use crate::{
    device_id::DeviceIdError, encode::CRC, InternalPacket, Packet, Payload, TcPacket, Timestamp,
    TmPacket, VERSION,
};

#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error(transparent)]
    Cobs(#[from] cobs::DecodeError),
    #[error("buffer too short to hold a complete packet ({0} bytes long)")]
    BufferTooShort(usize),
    #[error("unsupported protocol version ({0})")]
    UnsupportedVersion(u8),
    #[error("invalid packet checksum (expected {expected}, found {found})")]
    InvalidChecksum { expected: u16, found: u16 },
    #[error("invalid packet length (expected {expected}, found {found})")]
    InvalidLength { expected: usize, found: usize },
    #[error(transparent)]
    IdError(#[from] DeviceIdError),
}

impl Packet {
    pub fn decode_single(buf: &mut [u8]) -> Result<Self, DecodeError> {
        let len = cobs::decode_in_place(buf)?;

        if len < 13 {
            return Err(DecodeError::BufferTooShort(len));
        }

        if buf[0] != VERSION {
            return Err(DecodeError::UnsupportedVersion(buf[0]));
        }

        let found_checksum = u16::from_le_bytes([buf[len - 2], buf[len - 1]]);
        let expected_checksum = CRC.checksum(&buf[..len - 2]);
        if found_checksum != expected_checksum {
            return Err(DecodeError::InvalidChecksum {
                expected: expected_checksum,
                found: found_checksum,
            });
        }

        let found_payload_len = buf[1] as usize;
        let expected_payload_len = len - 13;
        if found_payload_len != expected_payload_len {
            return Err(DecodeError::InvalidLength {
                expected: expected_payload_len,
                found: found_payload_len,
            });
        }

        let tmtc = (buf[2] & 1 << 7) == 0;
        let id = (buf[2] & 0b01111100) >> 2;
        // A range can't be used here because from_le_bytes expects a [u8; 8]
        let timestamp = u64::from_le_bytes([
            buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10],
        ]);

        let packet = InternalPacket::new(
            id.try_into()?,
            Timestamp::new(timestamp),
            // Unwrapping is safe here because len is at most 255, so the slice
            // is never too long for Payload
            Payload::from_raw_bytes(&buf[11..len - 2]).unwrap(),
        );

        Ok(if tmtc {
            Self::TmPacket(TmPacket(packet))
        } else {
            Self::TcPacket(TcPacket(packet))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{DeviceId, Packet, VERSION};

    #[test]
    fn internal_packet_decode_tm_packet_works() {
        let mut buf = [
            5, VERSION, 4, 4, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 3, 28, 228, 0,
        ];

        let packet = Packet::decode_single(&mut buf).unwrap();

        let Packet::TmPacket(packet) = packet else {
            panic!("Decoded packet is not TmPacket")
        };
        assert_eq!(packet.version(), VERSION);
        assert_eq!(packet.device_id(), &DeviceId::System);
        assert_eq!(packet.timestamp().get(), 10);
        assert_eq!(packet.payload().as_bytes(), [0xEF, 0xCD, 0xAB, 0]);
    }
}
