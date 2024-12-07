use smallvec::SmallVec;

use crate::{Element, LayoutCtx, Node, ParentLayout, Tree, Widget};

pub struct TriLayout<MSG> {
    children: SmallVec<[Element<MSG>; 4]>,
}

impl<MSG> Default for TriLayout<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG> TriLayout<MSG> {
    pub fn new() -> Self {
        let mut children = SmallVec::new();

        children.push(Element::null());
        children.push(Element::null());
        children.push(Element::null());

        Self { children }
    }

    pub fn start(mut self, widget: impl Into<Element<MSG>>) -> Self {
        self.children[0] = widget.into();
        self
    }

    pub fn center(mut self, widget: impl Into<Element<MSG>>) -> Self {
        self.children[1] = widget.into();
        self
    }

    pub fn end(mut self, widget: impl Into<Element<MSG>>) -> Self {
        self.children[2] = widget.into();
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

impl<MSG> Widget<MSG> for TriLayout<MSG> {
    type State = ();

    fn children(&self) -> &[Element<MSG>] {
        &self.children
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        let start = self.children[0].as_widget().layout(
            &mut tree.children[0],
            &ParentLayout {
                x: parent.x,
                y: parent.y,
                w: parent.w,
                h: parent.h,
            },
            ctx,
        );

        let center = {
            let mut node = self.children[1].as_widget().layout(
                &mut tree.children[1],
                &ParentLayout {
                    x: parent.x,
                    y: parent.y,
                    w: parent.w,
                    h: parent.h,
                },
                ctx,
            );

            let x_offset = parent.w / 2.0 - node.w / 2.0;
            node.for_each_descend_mut(&|node| {
                node.x += x_offset;
            });

            node
        };

        let end = {
            let mut node = self.children[2].as_widget().layout(
                &mut tree.children[2],
                &ParentLayout {
                    x: parent.x,
                    y: parent.y,
                    w: parent.w,
                    h: parent.h,
                },
                ctx,
            );

            let x_offset = parent.w - node.w;
            node.for_each_descend_mut(&|node| {
                node.x += x_offset;
            });

            node
        };

        Node {
            x: parent.x,
            y: parent.y,
            w: parent.w,
            h: parent.h,
            children: vec![start, center, end],
        }
    }
}

impl<MSG: 'static> From<TriLayout<MSG>> for Element<MSG> {
    fn from(value: TriLayout<MSG>) -> Self {
        Element::new(value)
    }
}
