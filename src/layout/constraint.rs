use super::util::size::Size;

#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    min: Size,
    pref: Size,
    max: Size,
}

impl Constraint {
    pub fn new(pref: Size) -> Self {
        Self {
            min: Size::ZERO,
            pref,
            max: Size::INFINITY,
        }
    }

    pub fn with_min(self, min: Size) -> Self {
        Self {
            min,
            pref: self.pref,
            max: self.max,
        }
    }

    pub fn with_max(self, max: Size) -> Self {
        Self {
            min: self.min,
            pref: self.pref,
            max,
        }
    }

    pub fn width(&self, width: f32) -> f32 {
        self.min
            .width
            .max(width)
            .min(self.max.width)
            .max(self.pref.width)
    }

    pub fn height(&self, height: f32) -> f32 {
        self.min
            .height
            .max(height)
            .min(self.max.height)
            .max(self.pref.height)
    }
}

#[cfg(test)]
mod test {
    use crate::layout::util::size::Size;

    use super::Constraint;

    #[test]
    pub fn constraint_width() {
        let constraint = Constraint::new(Size {
            width: 800.0,
            height: 600.0,
        })
        .with_min(Size {
            width: 200.0,
            height: 100.0,
        })
        .with_max(Size {
            width: 1200.0,
            height: 800.0,
        });
        assert_eq!(constraint.width(1500.0), 1200.0);
        assert_eq!(constraint.width(100.0), 800.0);
        assert_eq!(constraint.width(600.0), 800.0);
    }
}
