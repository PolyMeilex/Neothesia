use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::{Element, WidgetAny};

pub struct UnknownState;
struct NullTreeState;

#[track_caller]
fn type_check_assert<NEW: Any>(state_type_id: TypeId) {
    if TypeId::of::<NEW>() != TypeId::of::<UnknownState>() {
        assert_eq!(TypeId::of::<NEW>(), state_type_id);
    } else {
        // Always allow cast to unknown
    }
}

pub struct TreeState<T = UnknownState> {
    state: Box<dyn Any>,
    _ph: PhantomData<T>,
}

impl<T: Any> TreeState<T> {
    pub(crate) fn new(state: impl Any) -> Self {
        Self {
            state: Box::new(state),
            _ph: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        self.state.downcast_ref::<T>().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.state.downcast_mut::<T>().unwrap()
    }

    fn state_type_id(&self) -> TypeId {
        self.state.as_ref().type_id()
    }

    fn cast<NEW: Any>(self) -> TreeState<NEW> {
        type_check_assert::<NEW>(self.state_type_id());
        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }
}

pub struct Tree<T = UnknownState> {
    pub state: TreeState<T>,
    pub children: Vec<Tree>,
    _ph: PhantomData<T>,
}

impl<T: 'static> Tree<T> {
    #[expect(unused)]
    pub(crate) fn cast<NEW: Any>(self) -> Tree<NEW> {
        type_check_assert::<NEW>(self.state.state_type_id());
        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }

    pub(crate) fn cast_ref<NEW: Any>(&self) -> &Tree<NEW> {
        type_check_assert::<NEW>(self.state.state_type_id());
        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }

    pub(crate) fn cast_mut<NEW: Any>(&mut self) -> &mut Tree<NEW> {
        type_check_assert::<NEW>(self.state.state_type_id());
        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }

    pub fn null() -> Self {
        Self {
            state: TreeState::new(Box::new(NullTreeState)),
            children: Vec::new(),
            _ph: PhantomData,
        }
    }

    pub fn new<MSG>(widget: &dyn WidgetAny<MSG>) -> Self {
        let children = widget
            .children()
            .iter()
            .map(|w| Tree::new(w.as_widget()))
            .collect();

        Self {
            state: widget.state().cast(),
            children,
            _ph: PhantomData,
        }
    }

    pub fn diff<MSG>(&mut self, new: &dyn WidgetAny<MSG>) {
        if self.state.state_type_id() == new.state_type_id() {
            self.diff_children(new.children());
        } else {
            *self = Self::new(new);
        }
    }

    fn diff_children<MSG>(&mut self, new_children: &[Element<MSG>]) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        for (tree, widget) in self.children.iter_mut().zip(new_children.iter()) {
            tree.diff(widget.as_widget());
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..]
                    .iter()
                    .map(|widget| Tree::new(widget.as_widget())),
            );
        }
    }
}
