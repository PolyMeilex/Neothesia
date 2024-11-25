use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::{Element, WidgetAny};

pub struct UnknownState;
struct NullTreeState;

pub struct Tree<T = UnknownState> {
    pub state: Box<dyn Any>,
    pub children: Vec<Tree>,
    _ph: PhantomData<T>,
}

impl<T: 'static> Tree<T> {
    pub fn remap<NEW: Any>(self) -> Tree<NEW> {
        if TypeId::of::<NEW>() != TypeId::of::<UnknownState>() {
            assert_eq!(TypeId::of::<NEW>(), self.state_type_id());
        }

        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }

    pub fn remap_ref<NEW: Any>(&self) -> &Tree<NEW> {
        if TypeId::of::<NEW>() != TypeId::of::<UnknownState>() {
            assert_eq!(TypeId::of::<NEW>(), self.state_type_id());
        }

        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }

    pub fn remap_mut<NEW: Any>(&mut self) -> &mut Tree<NEW> {
        if TypeId::of::<NEW>() != TypeId::of::<UnknownState>() {
            assert_eq!(TypeId::of::<NEW>(), self.state_type_id());
        }

        // SAFTETY: T is only in phantom data
        unsafe { std::mem::transmute(self) }
    }

    pub fn state(&self) -> &T {
        self.state.downcast_ref::<T>().unwrap()
    }

    pub fn state_mut(&mut self) -> &mut T {
        self.state.downcast_mut::<T>().unwrap()
    }

    pub fn null() -> Self {
        Self {
            state: Box::new(NullTreeState),
            children: Vec::new(),
            _ph: PhantomData,
        }
    }

    pub fn new<MSG>(widget: &dyn WidgetAny<MSG>) -> Self {
        Self {
            state: widget.state(),
            children: widget.children(),
            _ph: PhantomData,
        }
    }

    pub fn diff<MSG>(&mut self, new: &dyn WidgetAny<MSG>) {
        if self.state_type_id() == new.state_type_id() {
            new.diff(self.remap_mut());
        } else {
            *self = Self::new(new);
        }
    }

    pub fn diff_children<MSG>(&mut self, new_children: &[Element<MSG>]) {
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
                    .map(|widget| Self::new(widget.as_widget()).remap()),
            );
        }
    }

    pub fn state_type_id(&self) -> TypeId {
        self.state.as_ref().type_id()
    }
}
