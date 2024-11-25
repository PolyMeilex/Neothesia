use smallvec::SmallVec;

use crate::{Element, Widget};

pub struct Stack<MSG> {
    children: SmallVec<[Element<MSG>; 4]>,
}

impl<MSG> Default for Stack<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> Stack<MSG> {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }

    pub fn push(mut self, widget: impl Into<Element<MSG>>) -> Self {
        self.children.push(widget.into());
        self
    }

    pub fn when(self, v: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if v {
            f(self)
        } else {
            self
        }
    }
}

impl<MSG> Widget<MSG> for Stack<MSG> {
    type State = ();

    fn children(&self) -> &[Element<MSG>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Element<MSG>] {
        &mut self.children
    }
}

impl<MSG: 'static> From<Stack<MSG>> for Element<MSG> {
    fn from(value: Stack<MSG>) -> Self {
        Element::new(value)
    }
}
