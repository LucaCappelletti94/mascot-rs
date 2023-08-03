pub trait Zero {
    const ZERO: Self;
}

impl Zero for u8 {
    const ZERO: Self = 0;
}

impl Zero for u16 {
    const ZERO: Self = 0;
}

impl Zero for u32 {
    const ZERO: Self = 0;
}

impl Zero for u64 {
    const ZERO: Self = 0;
}

impl Zero for u128 {
    const ZERO: Self = 0;
}

impl Zero for usize {
    const ZERO: Self = 0;
}