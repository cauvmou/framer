use super::{point::Point, size::Size};

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn center(&self) -> Point {
        (self.x + self.width / 2.0, self.y + self.height / 2.0).into()
    }

    pub fn recenter(&mut self, point: &Point) {
        self.x = point.x - self.width / 2.0;
        self.y = point.y - self.height / 2.0;
    }
}
