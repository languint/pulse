use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct AnimationTokens {
    pub instant: Duration,
    pub fast: Duration,
    pub normal: Duration,
    pub slow: Duration,
}
impl AnimationTokens {
    pub const DEFAULT: Self = Self {
        instant: Duration::from_millis(0),
        fast: Duration::from_millis(100),
        normal: Duration::from_millis(200),
        slow: Duration::from_millis(350),
    };
}
