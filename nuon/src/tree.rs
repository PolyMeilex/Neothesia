use std::any::{Any, TypeId};

use crate::{Element, WidgetAny};

pub struct Tree {
    pub state: Box<dyn Any>,
    pub children: Vec<Tree>,
}

impl Tree {
    pub fn null() -> Self {
        Self {
            state: Box::new(()),
            children: Vec::new(),
        }
    }

    pub fn new<MSG>(widget: &dyn WidgetAny<MSG>) -> Self {
        Self {
            state: widget.default_state(),
            children: widget.children(),
        }
    }

    pub fn diff<MSG>(&mut self, new: &dyn WidgetAny<MSG>) {
        if self.state_type_id() == new.state_type_id() {
            new.diff(self);
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
                    .map(|widget| Self::new(widget.as_widget())),
            );
        }
    }

    // TODO: remove
    pub fn diff_children2<MSG>(&mut self, new_children: &[&Element<MSG>]) {
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
                    .map(|widget| Self::new(widget.as_widget())),
            );
        }
    }

    // TODO: remove
    pub fn diff_children3<MSG>(&mut self, new_children: &[&dyn WidgetAny<MSG>]) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        for (tree, widget) in self.children.iter_mut().zip(new_children.iter()) {
            tree.diff(*widget);
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..]
                    .iter()
                    .map(|widget| Self::new(*widget)),
            );
        }
    }

    pub fn state_type_id(&self) -> TypeId {
        self.state.as_ref().type_id()
    }
}
