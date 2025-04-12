/// Data contained inside a packet
///
/// TODO: Make the payload generic
pub trait Payload: bytemuck::NoUninit {
    const SIZE: usize = core::mem::size_of::<Self>();
    const SIZE_BOUND: () = assert!(Self::SIZE < 256, "Payload size must be less than 256 bytes");

    fn to_le_bytes(&self) -> &[u8] {
        // Make sure the const assertion is evaluated at compile time
        // This will result in a compile-time error when trying to convert
        // a type larger than 255 bytes to a byte slice
        #[allow(clippy::let_unit_value)]
        let _ = Self::SIZE_BOUND;
        bytemuck::bytes_of(self)
    }
}

// Blanket implementation for arrays of u8 with lengths 1 to 255
impl<const N: usize> Payload for [u8; N]
where
    [u8; N]: bytemuck::NoUninit,
{
    const SIZE: usize = N;
}

impl Payload for u8 {}
impl Payload for u16 {}
impl Payload for u32 {}
impl Payload for u64 {}
impl Payload for u128 {}

impl Payload for i8 {}
impl Payload for i16 {}
impl Payload for i32 {}
impl Payload for i64 {}
impl Payload for i128 {}
