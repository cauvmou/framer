use crate::{layout::util::size::Size, util};

use super::{node::LayoutNode, util::rectangle::Rectangle};

pub trait LayoutStrategy {
    fn apply(&self, node: &mut LayoutNode);
}

#[derive(Clone)]
pub struct BlockLayout {}

impl LayoutStrategy for BlockLayout {
    fn apply(&self, node: &mut LayoutNode) {
        // Scale
        let node_width = node
            .children
            .iter()
            .map(|node| node.area.width)
            .reduce(f32::max)
            .unwrap_or(0.0);
        node.area.width = node.element.constraints.width(node_width);
        let sum_height: f32 = node.children.iter().map(|node| node.area.height).sum();
        node.area.height = node.element.constraints.height(sum_height);

        // Position
        let mut cursor = node.area.y;
        node.children.iter_mut().for_each(|node| {
            node.area.x = node.area.x;
            node.area.y = cursor;
            cursor += node.area.height;
        });
    }
}

#[derive(Clone)]
pub struct FlexLayout {}

impl LayoutStrategy for FlexLayout {
    fn apply(&self, node: &mut LayoutNode) {
        todo!()
    }
}
