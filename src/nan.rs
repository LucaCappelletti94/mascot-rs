pub trait NaN {
    /// Returns whether the current float does not represent a number.
    fn is_nan(&self) -> bool;
}

impl NaN for f32 {
    fn is_nan(&self) -> bool {
        f32::is_nan(*self)
    }
}

impl NaN for f64 {
    fn is_nan(&self) -> bool {
        f64::is_nan(*self)
    }
}

impl NaN for str {
    /// Returns whether the current string does not represent a number.
    fn is_nan(&self) -> bool {
        self.trim().is_empty()
            || self == "N/A"
            || self == "NaN"
            || self == "nan"
            || self == "NAN"
            || self == "n/a"
            || self == "na"
            || self == "NA"
    }
}
