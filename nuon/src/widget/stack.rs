use smallvec::SmallVec;

use super::base::layout::GenericLayout;
use crate::{Element, Node};

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

impl<MSG: 'static> From<Stack<MSG>> for Element<MSG> {
    fn from(value: Stack<MSG>) -> Self {
        let base =
            GenericLayout::<_, MSG>::new(value.children, move |widgets, tree, parent, ctx| {
                let mut children = Vec::with_capacity(widgets.len());

                for (ch, tree) in widgets.iter().zip(tree.children.iter_mut()) {
                    children.push(ch.as_widget().layout(tree, parent, ctx));
                }

                Node {
                    x: parent.x,
                    y: parent.y,
                    w: parent.w,
                    h: parent.h,
                    children,
                }
            });

        Element::new(base)
    }
}
