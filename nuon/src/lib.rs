pub use euclid;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

pub use elements_map::{Element, ElementBuilder, ElementId, ElementsMap};
mod elements_map {
    use crate::{Point, Rect, Size};

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ElementId(thunderdome::Index);

    pub struct ElementBuilder<M> {
        name: Option<&'static str>,
        on_click: Option<M>,
        rect: Rect,
    }

    impl<M> Default for ElementBuilder<M> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<M> ElementBuilder<M> {
        pub fn new() -> Self {
            Self {
                name: None,
                on_click: None,
                rect: Rect::zero(),
            }
        }

        pub fn name(mut self, name: &'static str) -> Self {
            self.name = Some(name);
            self
        }

        pub fn on_click(mut self, msg: M) -> Self {
            self.on_click = Some(msg);
            self
        }

        pub fn rect(mut self, rect: Rect) -> Self {
            self.rect = rect;
            self
        }

        pub fn position(mut self, pos: Point) -> Self {
            self.rect.origin = pos;
            self
        }

        pub fn size(mut self, size: Size) -> Self {
            self.rect.size = size;
            self
        }

        fn build(self) -> Element<M> {
            Element {
                name: self.name.unwrap_or("Element"),
                on_click: self.on_click,
                hovered: false,
                rect: self.rect,
            }
        }
    }

    #[derive(Debug)]
    pub struct Element<M> {
        name: &'static str,
        on_click: Option<M>,
        hovered: bool,
        rect: Rect,
    }

    impl<M> Element<M> {
        pub fn name(&self) -> &'static str {
            self.name
        }

        pub fn hovered(&self) -> bool {
            self.hovered
        }

        pub fn on_click(&self) -> Option<&M> {
            self.on_click.as_ref()
        }
    }

    #[derive(Debug, Default)]
    pub struct ElementsMap<M> {
        elements: thunderdome::Arena<Element<M>>,
        hovered: Option<ElementId>,
    }

    impl<M> ElementsMap<M> {
        pub fn new() -> Self {
            Self {
                elements: thunderdome::Arena::new(),
                hovered: None,
            }
        }

        pub fn insert(&mut self, builder: ElementBuilder<M>) -> ElementId {
            let id = ElementId(self.elements.insert(builder.build()));
            self.listen_for_mouse(id);
            id
        }

        pub fn update(&mut self, id: ElementId, rect: Rect) {
            let Some(element) = self.elements.get_mut(id.0) else {
                // TODO: make this debug panic with a log
                panic!("Element not found");
            };

            element.rect = rect;
        }

        fn listen_for_mouse(&mut self, _id: ElementId) {
            // TODO: Track stacking order in a vec
        }

        pub fn get(&self, id: ElementId) -> Option<&Element<M>> {
            self.elements.get(id.0)
        }

        pub fn update_cursor_pos(&mut self, point: Point) {
            self.hovered = None;
            for (id, element) in self.elements.iter_mut() {
                element.hovered = element.rect.contains(point);
                if self.hovered.is_none() && element.hovered {
                    self.hovered = Some(ElementId(id));
                }
            }
        }

        pub fn hovered_element_id(&self) -> Option<ElementId> {
            self.hovered
        }

        pub fn hovered_element(&self) -> Option<(ElementId, &Element<M>)> {
            let id = self.hovered?;
            let element = self.elements.get(id.0)?;
            Some((id, element))
        }

        pub fn element_under(&self, point: Point) -> Option<ElementId> {
            for (id, element) in self.elements.iter() {
                if element.rect.contains(point) {
                    return Some(ElementId(id));
                }
            }

            None
        }
    }
}

pub use layout::{RowItem, RowLayout};
mod layout {
    use std::any::Any;

    use smallvec::SmallVec;

    #[derive(Debug)]
    pub struct RowItem {
        pub width: f32,
        pub x: f32,
    }

    #[derive(Debug)]
    pub struct RowLayout {
        items: SmallVec<[RowItem; 20]>,
        width: f32,
        gap: f32,
        dirty: bool,
        initialized: Option<Box<dyn Any>>,
    }

    impl Default for RowLayout {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RowLayout {
        pub fn new() -> Self {
            Self {
                items: SmallVec::new(),
                width: 0.0,
                gap: 0.0,
                dirty: true,
                initialized: None,
            }
        }

        pub fn set_gap(&mut self, gap: f32) {
            self.gap = gap;
        }

        pub fn once<T: Copy + 'static>(&mut self, cb: impl FnOnce(&mut Self) -> T) -> T {
            if self.initialized.is_none() {
                self.initialized = Some(Box::new(cb(self)));
            }

            let data = self.initialized.as_ref().unwrap();
            *data.downcast_ref().unwrap()
        }

        pub fn invalidate(&mut self) {
            self.dirty = true;
        }

        pub fn push(&mut self, width: f32) -> usize {
            let id = self.items.len();
            self.items.push(RowItem { width, x: 0.0 });
            id
        }

        pub fn width(&self) -> f32 {
            self.width
        }

        pub fn items(&self) -> &[RowItem] {
            &self.items
        }

        pub fn resolve_left(&mut self, origin: f32) {
            if !self.dirty {
                return;
            }
            self.dirty = false;

            let mut x = origin;

            for item in self.items.iter_mut() {
                item.x = x;
                x += item.width + self.gap;
            }

            self.width = self
                .items
                .last()
                .map(|i| (i.x - origin) + i.width)
                .unwrap_or(0.0);
        }

        pub fn resolve_right(&mut self, origin: f32) {
            if !self.dirty {
                return;
            }
            self.dirty = false;

            let mut x = origin;

            for item in self.items.iter_mut() {
                x -= item.width;
                item.x = x;
                x -= self.gap;
            }

            self.width = self
                .items
                .last()
                .map(|i| (i.x - origin) - i.width)
                .unwrap_or(0.0)
                .abs();
        }
    }
}
