use smallvec::SmallVec;

use crate::{Element, LayoutCtx, Node, ParentLayout, Tree, Widget};

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

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        stack_layout(self, tree, parent, ctx)
    }
}

pub fn stack_layout<MSG, W: Widget<MSG> + ?Sized>(
    this: &W,
    tree: &mut Tree<W::State>,
    parent: &ParentLayout,
    ctx: &LayoutCtx,
) -> Node {
    let widgets = this.children();
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
}

impl<MSG: 'static> From<Stack<MSG>> for Element<MSG> {
    fn from(value: Stack<MSG>) -> Self {
        Element::new(value)
    }
}
