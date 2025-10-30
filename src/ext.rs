pub trait ToARGB {
    fn to_argb(&self) -> u32;
}

impl ToARGB for u8 {
    #[inline]
    fn to_argb(&self) -> u32 {
        match self {
            1u8 => u32::MAX,
            _ => 0u32,
        }
    }
}
