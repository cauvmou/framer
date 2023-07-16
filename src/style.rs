// Every style property a element could have.
pub struct Style {
    margin: QuadValue<Unit>,
    padding: QuadValue<Unit>,
    border: QuadValue<Unit>,
}

#[derive(Debug, Clone, Copy)]
pub enum Unit {
    Percent(f32),
    Pixel(u32),
}

// Util for Margins, Padding, etc...
#[derive(Debug, Clone, Copy)]
pub struct QuadValue<T: Copy> {
    left: T,
    top: T,
    right: T,
    bottom: T,
}

impl<T: Copy> QuadValue<T> {}

impl<T: Copy> From<T> for QuadValue<T> {
    fn from(value: T) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }
}

impl<T: Copy> From<(T, T)> for QuadValue<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            left: value.0,
            top: value.1,
            right: value.0,
            bottom: value.1,
        }
    }
}

impl<T: Copy> From<[T; 2]> for QuadValue<T> {
    fn from(value: [T; 2]) -> Self {
        Self {
            left: value[0],
            top: value[1],
            right: value[0],
            bottom: value[1],
        }
    }
}

impl<T: Copy> From<(T, T, T, T)> for QuadValue<T> {
    fn from(value: (T, T, T, T)) -> Self {
        Self {
            left: value.0,
            top: value.1,
            right: value.2,
            bottom: value.3,
        }
    }
}

impl<T: Copy> From<[T; 4]> for QuadValue<T> {
    fn from(value: [T; 4]) -> Self {
        Self {
            left: value[0],
            top: value[1],
            right: value[2],
            bottom: value[3],
        }
    }
}
