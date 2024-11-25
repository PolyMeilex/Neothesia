use crate::{Element, Widget};

#[derive(Default, Debug)]
pub struct Null;

impl Null {
    pub fn new() -> Self {
        Self {}
    }
}

impl<MSG> Widget<MSG> for Null {
    type State = ();
}

impl<MSG> From<Null> for Element<MSG> {
    fn from(value: Null) -> Self {
        Element::new(value)
    }
}
