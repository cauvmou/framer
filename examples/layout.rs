use framer::{
    component::Element,
    layout::{constraint::Constraint, strategy::BlockLayout, util::size::Size},
};

pub fn main() {
    let constraint1 = Constraint::new(Size {
        width: 800.0,
        height: 200.0,
    });
    let constraint2 = Constraint::new(Size {
        width: 600.0,
        height: 150.0,
    });
    let constraint3 = Constraint::new(Size {
        width: 1200.0,
        height: 300.0,
    });

    let element = Element::new(&BlockLayout {})
        .with_constraints(constraint1)
        .add_child(Element::new(&BlockLayout {}).with_constraints(constraint2).add_child(Element::new(&BlockLayout {}).with_constraints(constraint3)))
        .add_child(Element::new(&BlockLayout {}).with_constraints(constraint2))
        .add_child(Element::new(&BlockLayout {}).with_constraints(constraint2));

    let mut node = element.layout();
    node.format();
    let views = node.flatten();
    println!("{views:#?}");
}
