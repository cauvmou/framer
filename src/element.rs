use std::{cell::RefCell, rc::Rc};

use crate::layout::{
    constraint::Constraint, node::LayoutNode, strategy::LayoutStrategy, util::size::Size,
};

#[derive(Clone)]
pub struct Element<'e> {
    children: Vec<Element<'e>>,
    pub constraints: Constraint,
    pub strategy: &'e dyn LayoutStrategy,
}

impl<'e> Element<'e> {
    pub fn new(strategy: &'e dyn LayoutStrategy) -> Self {
        Self {
            children: Vec::new(),
            constraints: Constraint::new(Size::ZERO),
            strategy,
        }
    }

    pub fn with_constraints(mut self, constraints: Constraint) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn add_child(mut self, element: Element<'e>) -> Self {
        self.children.push(element);
        self
    }

    pub fn layout(&self) -> LayoutNode {
        let children = self
            .children
            .iter()
            .map(|child| child.layout())
            .collect::<Vec<LayoutNode>>();

        LayoutNode::new(self, children)
    }
}
