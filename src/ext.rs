pub trait ToARGB {
    fn to_argb(&self) -> u32;
}

impl ToARGB for u8 {
    fn to_argb(&self) -> u32 {
        match self {
            0u8 => 0u32,
            1u8 => u32::MAX,
            _ => 0u32
        }
    }
}