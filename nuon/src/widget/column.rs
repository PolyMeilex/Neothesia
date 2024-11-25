use smallvec::SmallVec;

use super::base::layout::GenericLayout;
use crate::{Element, Node, ParentLayout};

pub struct Column<MSG> {
    children: SmallVec<[Element<MSG>; 4]>,
    gap: f32,
}

impl<MSG> Default for Column<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> Column<MSG> {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            gap: 0.0,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
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

impl<MSG: 'static> From<Column<MSG>> for Element<MSG> {
    fn from(this: Column<MSG>) -> Self {
        let base =
            GenericLayout::<_, MSG>::new(this.children, move |widgets, tree, parent, ctx| {
                let mut children = Vec::with_capacity(widgets.len());

                let mut item_layout = ParentLayout {
                    x: parent.x,
                    y: parent.y,
                    w: parent.w,
                    h: parent.h,
                };

                let mut total_height = 0.0;

                for (ch, tree) in widgets.iter().zip(tree.children.iter_mut()) {
                    let node = ch.as_widget().layout(tree, &item_layout, ctx);

                    item_layout.y += node.h;
                    item_layout.h -= node.h;

                    item_layout.y += this.gap;
                    item_layout.h -= this.gap;

                    total_height += node.h;
                    total_height += this.gap;

                    children.push(node);
                }

                total_height -= this.gap;
                total_height = total_height.max(0.0);

                Node {
                    x: parent.x,
                    y: parent.y,
                    w: parent.w,
                    h: total_height,
                    children,
                }
            });

        Element::new(base)
    }
}
