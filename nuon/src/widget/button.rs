use crate::{
    Color, Element, Event, LayoutCtx, MouseButton, Node, ParentLayout, RenderCtx, Renderer, Tree,
    UpdateCtx, Widget,
};

#[derive(Default)]
pub struct ButtonState {
    is_hovered: bool,
    is_pressed: bool,
}

pub struct Button<MSG> {
    w: Option<f32>,
    h: Option<f32>,
    color: Color,
    hover_color: Color,
    preseed_color: Color,
    on_click: Option<MSG>,
    border_radius: [f32; 4],
    icon: Option<&'static str>,
}

impl<MSG: Clone> Default for Button<MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<MSG: Clone> Button<MSG> {
    pub fn new() -> Self {
        Self {
            w: None,
            h: None,
            color: Color::new_u8(0, 0, 0, 0.0),
            hover_color: Color::new_u8(57, 55, 62, 1.0),
            preseed_color: Color::new_u8(67, 65, 72, 1.0),
            on_click: None,
            border_radius: [5.0; 4],
            icon: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.w = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.h = Some(height);
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    pub fn hover_color(mut self, color: impl Into<Color>) -> Self {
        self.hover_color = color.into();
        self
    }

    pub fn preseed_color(mut self, color: impl Into<Color>) -> Self {
        self.preseed_color = color.into();
        self
    }

    pub fn border_radius(mut self, border_radius: [f32; 4]) -> Self {
        self.border_radius = border_radius;
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn on_click(mut self, msg: MSG) -> Self {
        self.on_click = Some(msg);
        self
    }

    pub fn on_click_maybe(mut self, msg: Option<MSG>) -> Self {
        self.on_click = msg;
        self
    }
}

impl<MSG: Clone> Widget<MSG> for Button<MSG> {
    type State = ButtonState;

    fn layout(
        &self,
        _tree: &mut Tree<Self::State>,
        parent: &ParentLayout,
        _ctx: &LayoutCtx,
    ) -> Node {
        Node {
            x: parent.x,
            y: parent.y,
            w: self.w.unwrap_or(parent.w),
            h: self.h.unwrap_or(parent.h),
            children: vec![],
        }
    }

    fn render(
        &self,
        renderer: &mut dyn Renderer,
        layout: &Node,
        tree: &Tree<Self::State>,
        _ctx: &RenderCtx,
    ) {
        let state = tree.state.get();

        renderer.rounded_quad(
            layout.x,
            layout.y,
            layout.w,
            layout.h,
            if state.is_pressed {
                self.preseed_color
            } else if state.is_hovered {
                self.hover_color
            } else {
                self.color
            },
            self.border_radius,
        );

        if let Some(icon) = self.icon {
            let icon_size = 20.0;
            renderer.icon(
                layout.x + (layout.w - icon_size) / 2.0,
                layout.y + (layout.h - icon_size) / 2.0,
                icon_size,
                icon,
            );
        }
    }

    fn update(
        &self,
        event: Event,
        layout: &Node,
        tree: &mut Tree<Self::State>,
        ctx: &mut UpdateCtx<MSG>,
    ) {
        let state = tree.state.get_mut();

        match event {
            Event::CursorMoved { position } => {
                state.is_hovered = layout.contains(position.x, position.y);
            }
            Event::MousePress {
                button: MouseButton::Left,
            } => {
                if state.is_hovered {
                    ctx.grab_mouse();
                    state.is_pressed = true;
                }
            }
            Event::MouseRelease {
                button: MouseButton::Left,
            } => {
                if state.is_pressed {
                    ctx.ungrab_mouse();
                }

                if state.is_hovered && state.is_pressed {
                    if let Some(msg) = self.on_click.clone() {
                        ctx.messages.push(msg);
                    }
                }

                state.is_pressed = false;
            }
            _ => {}
        }
    }
}

impl<MSG: Clone + 'static> From<Button<MSG>> for Element<MSG> {
    fn from(value: Button<MSG>) -> Self {
        Element::new(value)
    }
}
