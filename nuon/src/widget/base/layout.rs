use smallvec::SmallVec;

use crate::{
    tree::Tree, Element, Event, LayoutCtx, Node, ParentLayout, RenderCtx, Renderer, UpdateCtx,
    Widget,
};

pub struct GenericLayout<F, MSG> {
    children: SmallVec<[Element<MSG>; 4]>,
    layout: F,
}

impl<L, MSG> GenericLayout<L, MSG>
where
    L: Fn(&[Element<MSG>], &mut Tree, &ParentLayout, &LayoutCtx) -> Node,
{
    pub fn new(children: SmallVec<[Element<MSG>; 4]>, layout: L) -> Self {
        Self { children, layout }
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

impl<L, MSG> Widget<MSG> for GenericLayout<L, MSG>
where
    L: Fn(&[Element<MSG>], &mut Tree, &ParentLayout, &LayoutCtx) -> Node,
{
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
        (self.layout)(&self.children, tree.remap_mut(), parent, ctx)
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

impl<L, MSG: 'static> From<GenericLayout<L, MSG>> for Element<MSG>
where
    L: Fn(&[Element<MSG>], &mut Tree, &ParentLayout, &LayoutCtx) -> Node + 'static,
{
    fn from(value: GenericLayout<L, MSG>) -> Self {
        Element::new(value)
    }
}
