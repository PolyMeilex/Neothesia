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
        on_release: Option<M>,
        on_cursor_move: Option<M>,
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
                on_release: None,
                on_cursor_move: None,
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

        pub fn on_release(mut self, msg: M) -> Self {
            self.on_release = Some(msg);
            self
        }

        pub fn on_cursor_move(mut self, msg: M) -> Self {
            self.on_cursor_move = Some(msg);
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
                on_release: self.on_release,
                on_cursor_move: self.on_cursor_move,
                hovered: false,
                rect: self.rect,
            }
        }
    }

    #[derive(Debug)]
    pub struct Element<M> {
        name: &'static str,
        on_click: Option<M>,
        on_release: Option<M>,
        on_cursor_move: Option<M>,
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

        pub fn rect(&self) -> Rect {
            self.rect
        }

        pub fn on_click(&self) -> Option<&M> {
            self.on_click.as_ref()
        }

        pub fn on_release(&self) -> Option<&M> {
            self.on_release.as_ref()
        }

        pub fn on_cursor_move(&self) -> Option<&M> {
            self.on_cursor_move.as_ref()
        }
    }

    #[derive(Debug, Default)]
    pub struct ElementsMap<M> {
        elements: thunderdome::Arena<Element<M>>,
        zorder: Vec<ElementId>,
        hovered: Option<ElementId>,
        pressed: Option<ElementId>,
        mouse_grab: Option<ElementId>,
    }

    impl<M> ElementsMap<M> {
        pub fn new() -> Self {
            Self {
                elements: thunderdome::Arena::new(),
                zorder: Vec::new(),
                hovered: None,
                pressed: None,
                mouse_grab: None,
            }
        }

        pub fn insert(&mut self, builder: ElementBuilder<M>) -> ElementId {
            let id = ElementId(self.elements.insert(builder.build()));
            self.listen_for_mouse(id);
            id
        }

        pub fn update(&mut self, id: ElementId, rect: Rect) -> Option<&Element<M>> {
            let Some(element) = self.elements.get_mut(id.0) else {
                // TODO: make this debug panic with a log
                panic!("Element not found");
            };

            element.rect = rect;
            Some(element)
        }

        fn listen_for_mouse(&mut self, id: ElementId) {
            // TODO: Make this smarter
            self.zorder.push(id);
        }

        pub fn get(&self, id: ElementId) -> Option<&Element<M>> {
            self.elements.get(id.0)
        }

        pub fn update_cursor_pos(&mut self, point: Point) {
            self.hovered = None;

            for id in self.zorder.iter().rev() {
                let Some(element) = self.elements.get_mut(id.0) else {
                    continue;
                };

                element.hovered = element.rect.contains(point);
                if self.hovered.is_none() && element.hovered {
                    self.hovered = Some(*id);
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

        pub fn set_mouse_grab(&mut self, id: Option<ElementId>) {
            self.mouse_grab = id;
        }

        pub fn current_mouse_grab_id(&self) -> Option<ElementId> {
            self.mouse_grab
        }

        pub fn current_mouse_grab(&self) -> Option<(ElementId, &Element<M>)> {
            let id = self.mouse_grab?;
            let element = self.elements.get(id.0)?;
            Some((id, element))
        }

        pub fn on_press(&mut self) -> Option<(ElementId, &Element<M>)> {
            let id = self.hovered?;
            let element = self.elements.get(id.0)?;
            self.pressed = Some(id);
            Some((id, element))
        }

        pub fn on_release(&mut self) -> Option<(ElementId, &Element<M>)> {
            let id = self.pressed.take()?;
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

pub use row_layout::{RowItem, RowLayout};
mod row_layout {
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

        fn width(&self) -> f32 {
            self.width
        }

        pub fn items(&self) -> &[RowItem] {
            &self.items
        }

        pub fn resolve_left(&mut self, x: f32) {
            if !self.dirty {
                return;
            }
            self.dirty = false;

            let mut x = x;

            for item in self.items.iter_mut() {
                item.x = x;
                x += item.width + self.gap;
            }

            self.width = self
                .items
                .last()
                .map(|i| (i.x - x) + i.width)
                .unwrap_or(0.0);
        }

        pub fn resolve_center(&mut self, x: f32, width: f32) {
            self.resolve_left(x);
            let center_x = width / 2.0 - self.width() / 2.0;
            for item in self.items.iter_mut() {
                item.x += center_x;
            }
        }

        pub fn resolve_right(&mut self, width: f32) {
            if !self.dirty {
                return;
            }
            self.dirty = false;

            let mut x = width;

            for item in self.items.iter_mut().rev() {
                x -= item.width;
                item.x = x;
                x -= self.gap;
            }

            self.width = self
                .items
                .last()
                .map(|i| (i.x - width) - i.width)
                .unwrap_or(0.0)
                .abs();
        }
    }
}

pub use tri_row_layout::TriRowLayout;
mod tri_row_layout {
    use crate::RowLayout;

    #[derive(Debug)]
    pub struct TriRowLayout {
        pub start: RowLayout,
        pub center: RowLayout,
        pub end: RowLayout,
    }

    impl Default for TriRowLayout {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TriRowLayout {
        pub fn new() -> Self {
            Self {
                start: RowLayout::new(),
                center: RowLayout::new(),
                end: RowLayout::new(),
            }
        }

        pub fn invalidate(&mut self) {
            self.start.invalidate();
            self.center.invalidate();
            self.end.invalidate();
        }

        pub fn resolve(&mut self, x: f32, width: f32) {
            self.start.resolve_left(x);
            self.center.resolve_center(x, width);
            self.end.resolve_right(width);
        }
    }
}
