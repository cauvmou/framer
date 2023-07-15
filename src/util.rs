#[inline]
pub fn max(x: &[f32]) -> Option<f32> {
    x.iter().copied().reduce(f32::max)
}
