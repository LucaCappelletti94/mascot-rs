pub trait StrictlyPositive {
    fn is_strictly_positive(&self) -> bool;
}

impl StrictlyPositive for f32 {
    fn is_strictly_positive(&self) -> bool {
        *self > 0.0
    }
}

impl StrictlyPositive for f64 {
    fn is_strictly_positive(&self) -> bool {
        *self > 0.0
    }
}