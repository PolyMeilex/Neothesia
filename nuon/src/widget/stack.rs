use smallvec::SmallVec;

use crate::{
    Element, Event, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, Tree, UpdateCtx, Widget,
};

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

        for (ch, tree) in self.children.iter().zip(tree.children.iter_mut()) {
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
            .rev()
        {
            ch.as_widget_mut().update(event.clone(), layout, tree, ctx);

            if ctx.is_event_captured() {
                return;
            }
        }
    }
}

impl<MSG: 'static> From<Stack<MSG>> for Element<MSG> {
    fn from(value: Stack<MSG>) -> Self {
        Element::new(value)
    }
}
