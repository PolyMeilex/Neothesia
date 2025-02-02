use smallvec::SmallVec;

use crate::{Element, LayoutCtx, Node, ParentLayout, Tree, Widget};

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

impl<MSG> Widget<MSG> for Column<MSG> {
    type State = ();

    fn children(&self) -> &[Element<MSG>] {
        &self.children
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        column_layout(self, tree, parent, ctx, self.gap)
    }
}

pub fn column_layout<MSG, W: Widget<MSG> + ?Sized>(
    this: &W,
    tree: &mut Tree<W::State>,
    parent: &ParentLayout,
    ctx: &LayoutCtx,
    gap: f32,
) -> Node {
    let mut children = Vec::with_capacity(this.children().len());

    let mut item_layout = ParentLayout {
        x: parent.x,
        y: parent.y,
        w: parent.w,
        h: parent.h,
    };

    let mut total_height = 0.0;

    for (ch, tree) in this.children().iter().zip(tree.children.iter_mut()) {
        let node = ch.as_widget().layout(tree, &item_layout, ctx);

        item_layout.y += node.h;
        item_layout.h -= node.h;

        item_layout.y += gap;
        item_layout.h -= gap;

        total_height += node.h;
        total_height += gap;

        children.push(node);
    }

    total_height -= gap;
    total_height = total_height.max(0.0);

    Node {
        x: parent.x,
        y: parent.y,
        w: parent.w,
        h: total_height,
        children,
    }
}

impl<MSG: 'static> From<Column<MSG>> for Element<MSG> {
    fn from(value: Column<MSG>) -> Self {
        Element::new(value)
    }
}
