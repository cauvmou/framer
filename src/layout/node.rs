use crate::component::Element;

use super::util::{point::Point, rectangle::Rectangle, size::Size};

pub struct LayoutNode<'e> {
    pub area: Rectangle,
    pub z_index: usize,
    pub element: &'e Element<'e>,
    pub children: Vec<LayoutNode<'e>>,
}

impl<'e> LayoutNode<'e> {
    pub fn new(element: &'e Element<'e>, children: Vec<LayoutNode<'e>>) -> Self {
        Self {
            element,
            children,
            z_index: 0,
            area: Rectangle::new(Point::ORIGIN, Size::ZERO),
        }
    }

    pub fn format(&mut self) {
        self.children.iter_mut().for_each(|child| {
            child.z_index += self.z_index+1;
            child.format();
        });
        self.element.strategy.apply(self);
    }

    pub fn flatten(mut self) -> Vec<View<'e>> {
        let children = self.children;
        let mut cast = children.into_iter().flat_map(|node| node.flatten()).collect::<Vec<View<'e>>>();
        self.children = Vec::new();
        cast.push(self.into());
        cast.sort_by(|a, b| a.z_index.cmp(&b.z_index));
        cast
    }
}

impl<'e> std::fmt::Debug for LayoutNode<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutNode")
            .field("z-index", &self.z_index)
            .field("area", &self.area)
            .field("children", &self.children)
            .finish()
    }
}

pub struct View<'e> {
    element: &'e Element<'e>,
    area: Rectangle,
    z_index: usize,
}

impl<'e> From<LayoutNode<'e>> for View<'e> {
    fn from(value: LayoutNode<'e>) -> Self {
        Self {
            element: value.element,
            area: value.area,
            z_index: value.z_index,
        }
    }
}

impl<'e> std::fmt::Debug for View<'e> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("View")
            .field("z-index", &self.z_index)
            .field("area", &self.area)
            .finish()
    }
}