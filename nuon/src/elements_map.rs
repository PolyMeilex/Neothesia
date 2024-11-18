use crate::{Point, Rect, Size};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ElementId(thunderdome::Index);

pub struct ElementBuilder<M> {
    name: Option<&'static str>,
    on_click: Option<M>,
    on_pressed: Option<M>,
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
            on_pressed: None,
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

    pub fn on_pressed(mut self, msg: M) -> Self {
        self.on_pressed = Some(msg);
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
            id: ElementId(thunderdome::Index::DANGLING),
            name: self.name.unwrap_or("Element"),
            on_click: self.on_click,
            on_pressed: self.on_pressed,
            on_release: self.on_release,
            on_cursor_move: self.on_cursor_move,
            hovered: false,
            rect: self.rect,
        }
    }
}

#[derive(Debug)]
pub struct Element<M> {
    id: ElementId,
    name: &'static str,
    /// Button-like click, needs both press and release without focus loss
    on_click: Option<M>,
    on_pressed: Option<M>,
    on_release: Option<M>,
    on_cursor_move: Option<M>,
    hovered: bool,
    rect: Rect,
}

impl<M> Element<M> {
    pub fn id(&self) -> ElementId {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn hovered(&self) -> bool {
        self.hovered
    }

    pub fn set_pos(&mut self, pos: impl Into<Point>) {
        self.rect.origin = pos.into();
    }

    pub fn set_size(&mut self, size: impl Into<Size>) {
        self.rect.size = size.into();
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn on_click(&self) -> Option<&M> {
        self.on_click.as_ref()
    }

    pub fn on_pressed(&self) -> Option<&M> {
        self.on_pressed.as_ref()
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

    pub fn iter(&self) -> impl Iterator<Item = &Element<M>> {
        self.elements.iter().map(|(_id, element)| element)
    }

    pub fn insert(&mut self, builder: ElementBuilder<M>) -> ElementId {
        let id = ElementId(self.elements.insert(builder.build()));
        self.elements.get_mut(id.0).unwrap().id = id;
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

    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut Element<M>> {
        self.elements.get_mut(id.0)
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

    pub fn hovered_element(&self) -> Option<&Element<M>> {
        let id = self.hovered?;
        let element = self.elements.get(id.0)?;
        Some(element)
    }

    pub fn set_mouse_grab(&mut self, id: Option<ElementId>) {
        self.mouse_grab = id;
    }

    pub fn current_mouse_grab_id(&self) -> Option<ElementId> {
        self.mouse_grab
    }

    pub fn current_mouse_grab(&self) -> Option<&Element<M>> {
        let id = self.mouse_grab?;
        let element = self.elements.get(id.0)?;
        Some(element)
    }

    pub fn on_press(&mut self) -> Option<&Element<M>> {
        let id = self.hovered?;
        let element = self.elements.get(id.0)?;
        self.pressed = Some(id);
        Some(element)
    }

    pub fn on_release(&mut self) -> Option<&Element<M>> {
        let id = self.pressed.take()?;
        let element = self.elements.get(id.0)?;
        Some(element)
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
