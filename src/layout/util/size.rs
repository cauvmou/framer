#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size::new(0.0, 0.0);
    pub const INFINITY: Size = Size::new(f32::MAX, f32::MAX);

    #[inline]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<(f32, f32)> for Size {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<[f32; 2]> for Size {
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}
