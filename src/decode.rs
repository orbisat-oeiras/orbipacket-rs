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
    /// Decode a buffer containing a single packet.
    ///
    /// The input buffer will be used to construct an instance of [`Self`].
    /// Since the buffer is unstuffed in-place, it is mutated. Thus, the original
    /// encoded bytes cannot be recovered after decoding.
    ///
    /// # Errors
    /// An error variant is returned if the provided bytes do not constitute a valid packet.
    /// Namely, the following conditions result in errors:
    /// - the bytes are not a valid COBS frame;
    /// - the (unstuffed) buffer is shorter than 13 bytes;
    /// - the packet's version isn't supported;
    /// - the reported payload length doesn't match it's actual length;
    /// - the CRC checksum is incorrect;
    /// - the control byte cannot be properly parsed into a device ID.
    ///
    /// # Examples
    /// ```
    /// use orbipacket::{Packet, DeviceId};
    ///
    /// let mut buf = [
    ///     5, 1, 4, 4, 10, 1, 1, 1, 1, 1, 1, 4, 0xEF, 0xCD, 0xAB, 3, 28, 228, 0,
    /// ];
    ///
    /// let packet = Packet::decode_single(&mut buf)?;
    ///
    /// let Packet::TmPacket(packet) = packet else {
    ///     panic!("Decoded packet is not TmPacket")
    /// };
    /// assert_eq!(packet.version(), 1);
    /// assert_eq!(packet.device_id(), &DeviceId::System);
    /// assert_eq!(packet.timestamp().get(), 10);
    /// assert_eq!(packet.payload().as_bytes(), [0xEF, 0xCD, 0xAB, 0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn decode_single(buf: &mut [u8]) -> Result<Self, DecodeError> {
        let len = cobs::decode_in_place(buf)?;

        if len < 13 {
            return Err(DecodeError::BufferTooShort(len));
        }

        if buf[0] != VERSION {
            return Err(DecodeError::UnsupportedVersion(buf[0]));
        }

        let found_payload_len = buf[1] as usize;
        let expected_payload_len = len - InternalPacket::OVERHEAD;
        if found_payload_len != expected_payload_len {
            return Err(DecodeError::InvalidLength {
                expected: expected_payload_len,
                found: found_payload_len,
            });
        }

        let found_checksum = u16::from_le_bytes([buf[len - 2], buf[len - 1]]);
        let expected_checksum = CRC.checksum(&buf[..len - 2]);

        if found_checksum != expected_checksum {
            return Err(DecodeError::InvalidChecksum {
                expected: expected_checksum,
                found: found_checksum,
            });
        }

        let tmtc = (buf[2] & 1 << 7) == 0;
        let id = (buf[2] & 0b01111100) >> 2;
        // A range can't be used here because from_le_bytes expects a [u8; 8]
        let timestamp = u64::from_le_bytes([buf[3], buf[4], buf[5], buf[6], buf[7], 0, 0, 0]);

        let packet = InternalPacket::new(
            id.try_into()?,
            // Unwrapping is safe here because we just created the value from 5 bytes
            Timestamp::new(timestamp).unwrap(),
            // Unwrapping is safe here because found_payload_len is at most 255, so the slice
            // is never too long for Payload
            Payload::from_raw_bytes(&buf[8..][..found_payload_len]).unwrap(),
        );

        Ok(if tmtc {
            Self::TmPacket(TmPacket(packet))
        } else {
            Self::TcPacket(TcPacket(packet))
        })
    }

    pub fn decode_stateless<'a, 'b>(
        buf: &'a mut [u8],
        out: &'b mut [Self],
    ) -> Result<(&'a mut [u8], &'b mut [Self]), DecodeError> {
        let mut out_idx: usize = 0;

        while let Some(idx) = buf.iter().position(|&x| x == 0) {
            if out_idx >= out.len() {
                // Decrement out_idx so output subslice is correct
                out_idx -= 1;
                break;
            }

            out[out_idx] = Self::decode_single(&mut buf[..idx])?;
            out_idx += 1;
        }

        let trailing_range = if let Some(idx) = buf.iter().position(|&x| x == 0) {
            idx..
        } else {
            buf.len()..
        };

        Ok((&mut buf[trailing_range], &mut out[..out_idx]))
    }
}

#[cfg(test)]
mod test {
    use crate::{DeviceId, Packet, VERSION};

    #[test]
    fn tm_packet_decode_works() {
        let mut buf = [
            0x05, VERSION, 0x04, 0x04, 0x0a, 0x01, 0x01, 0x01, 0x04, 0xEF, 0xCD, 0xAB, 0x03, 0x7e,
            0x12, 0,
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
    #[test]
    fn tc_packet_decode_works() {
        let mut buf = [
            0x05, VERSION, 0x04, 0x84, 0x0a, 0x01, 0x01, 0x01, 0x04, 0xEF, 0xCD, 0xAB, 0x03, 0x014,
            0x022, 0,
        ];

        let packet = Packet::decode_single(&mut buf).unwrap();

        let Packet::TcPacket(packet) = packet else {
            panic!("Decoded packet is not TmPacket")
        };
        assert_eq!(packet.version(), VERSION);
        assert_eq!(packet.device_id(), &DeviceId::System);
        assert_eq!(packet.timestamp().get(), 10);
        assert_eq!(packet.payload().as_bytes(), [0xEF, 0xCD, 0xAB, 0]);
    }
}
