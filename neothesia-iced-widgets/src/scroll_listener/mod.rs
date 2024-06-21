use iced_core::{mouse::ScrollDelta, Length, Size, Theme, Widget};

use super::Element;

pub struct ScrollListener<'a, M> {
    content: Element<'a, M>,
    on_scroll: Box<dyn Fn(f32) -> M>,
}

impl<'a, M> ScrollListener<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>, on_scroll: impl Fn(f32) -> M + 'static) -> Self {
        Self {
            content: content.into(),
            on_scroll: Box::new(on_scroll),
        }
    }
}

impl<'a, M> Widget<M, Theme, super::Renderer> for ScrollListener<'a, M> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut iced_core::widget::Tree,
        renderer: &super::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        self.content.as_widget().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &iced_core::widget::Tree,
        renderer: &mut super::Renderer,
        theme: &Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport)
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> iced_core::widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<iced_core::widget::Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut iced_core::widget::Tree) {
        self.content.as_widget().diff(tree)
    }

    fn operate(
        &self,
        state: &mut iced_core::widget::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &super::Renderer,
        operation: &mut dyn iced_core::widget::Operation<M>,
    ) {
        self.content
            .as_widget()
            .operate(state, layout, renderer, operation)
    }

    fn on_event(
        &mut self,
        state: &mut iced_core::widget::Tree,
        event: iced_core::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &super::Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, M>,
        viewport: &iced_core::Rectangle,
    ) -> iced_core::event::Status {
        if let iced_core::event::Status::Captured = self.content.as_widget_mut().on_event(
            state,
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return iced_core::event::Status::Captured;
        }

        if let iced_core::Event::Mouse(iced_core::mouse::Event::WheelScrolled { delta }) = event {
            let bounds = layout.bounds();

            if cursor.is_over(bounds) {
                let (ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. }) = delta;

                if y.abs() != 0.0 {
                    let msg = (self.on_scroll)(y);
                    shell.publish(msg);
                    return iced_core::event::Status::Captured;
                }
            }
        }

        iced_core::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        state: &iced_core::widget::Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced_core::Rectangle,
        renderer: &super::Renderer,
    ) -> iced_core::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut iced_core::widget::Tree,
        layout: iced_core::Layout<'_>,
        renderer: &super::Renderer,
        translation: iced_core::Vector,
    ) -> Option<iced_core::overlay::Element<'b, M, Theme, super::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(state, layout, renderer, translation)
    }
}

impl<'a, M: 'a> From<ScrollListener<'a, M>> for Element<'a, M> {
    fn from(value: ScrollListener<'a, M>) -> Self {
        Self::new(value)
    }
}
