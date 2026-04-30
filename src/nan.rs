/// Detects whether a numeric value is NaN.
pub trait NaN {
    /// Returns whether the current float does not represent a number.
    fn is_nan(&self) -> bool;
}

impl NaN for f32 {
    fn is_nan(&self) -> bool {
        Self::is_nan(*self)
    }
}

impl NaN for f64 {
    fn is_nan(&self) -> bool {
        Self::is_nan(*self)
    }
}
