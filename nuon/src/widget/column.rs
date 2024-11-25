use smallvec::SmallVec;

use crate::{
    tree::Tree, Element, Event, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, UpdateCtx,
    Widget,
};

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

    fn children(&self) -> Vec<Tree> {
        self.children
            .iter()
            .map(|w| Tree::new(w.as_widget()))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(self.children.as_ref());
    }

    fn layout(&self, tree: &mut Tree<Self::State>, parent: &ParentLayout, ctx: &LayoutCtx) -> Node {
        let mut children = Vec::with_capacity(self.children.len());

        let mut item_layout = ParentLayout {
            x: parent.x,
            y: parent.y,
            w: parent.w,
            h: parent.h,
        };

        let mut total_height = 0.0;

        for (ch, tree) in self.children.iter().zip(tree.children.iter_mut()) {
            let node = ch.as_widget().layout(tree, &item_layout, ctx);

            item_layout.y += node.h;
            item_layout.h -= node.h;

            item_layout.y += self.gap;
            item_layout.h -= self.gap;

            total_height += node.h;
            total_height += self.gap;

            children.push(node);
        }

        total_height -= self.gap;
        total_height = total_height.max(0.0);

        Node {
            x: parent.x,
            y: parent.y,
            w: parent.w,
            h: total_height,
            children,
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        ctx: &RenderCtx,
    ) {
        for ((ch, layout), tree) in self
            .children
            .iter()
            .zip(layout.children.iter())
            .zip(tree.children.iter())
        {
            ch.as_widget().render(renderer, layout, tree, ctx);
        }
    }

    fn update(
        &mut self,
        event: Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        for ((ch, layout), tree) in self
            .children
            .iter_mut()
            .zip(layout.children.iter())
            .zip(tree.children.iter_mut())
        {
            ch.as_widget_mut().update(event.clone(), layout, tree, ctx);
        }
    }
}

impl<MSG: 'static> From<Column<MSG>> for Element<MSG> {
    fn from(value: Column<MSG>) -> Self {
        Element::new(value)
    }
}
