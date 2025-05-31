use iced_core::{mouse, Event, Size, Theme};
use iced_graphics::core::Color;
use iced_runtime::user_interface::{self, UserInterface};
use iced_runtime::Task;

use super::{iced_clipboard::DummyClipboard, iced_conversion};
use crate::context::Context;

pub type Element<'a, M> = iced_core::Element<'a, M, Theme, iced_wgpu::Renderer>;

/// The core of a user interface application following The Elm Architecture.
pub trait Program: Sized {
    /// The type of __messages__ your [`Program`] will produce.
    type Message: std::fmt::Debug + Send;

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    fn update(&mut self, ctx: &mut Context, message: Self::Message) -> Task<Self::Message>;

    /// Returns the widgets to display in the [`Program`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(&self, ctx: &Context) -> Element<'_, Self::Message>;

    fn mouse_input(
        &self,
        _event: &iced_core::mouse::Event,
        _ctx: &Context,
    ) -> Option<Self::Message> {
        None
    }

    fn keyboard_input(
        &self,
        _event: &iced_core::keyboard::Event,
        _ctx: &Context,
    ) -> Option<Self::Message> {
        None
    }

    fn tick(&mut self, ctx: &mut Context);
}

/// The execution state of a [`Program`]. It leverages caching, event
/// processing, and rendering primitive storage.
pub struct State<P>
where
    P: Program + 'static,
{
    program: P,
    cache: Option<user_interface::Cache>,
    queued_events: Vec<Event>,
    queued_messages: Vec<P::Message>,
}

impl<P> State<P>
where
    P: Program + 'static,
{
    /// Creates a new [`State`] with the provided [`Program`], initializing its
    /// primitive with the given logical bounds and renderer.
    pub fn new(program: P, bounds: Size, ctx: &mut Context) -> Self {
        let user_interface = UserInterface::build(
            program.view(ctx),
            bounds,
            user_interface::Cache::default(),
            &mut ctx.iced_manager.renderer,
        );

        let cache = Some(user_interface.into_cache());

        State {
            program,
            cache,
            queued_events: Vec::new(),
            queued_messages: Vec::new(),
        }
    }

    /// Returns a reference to the [`Program`] of the [`State`].
    pub fn program(&self) -> &P {
        &self.program
    }

    /// Queues an event in the [`State`] for processing during an [`update`].
    ///
    /// [`update`]: Self::update
    pub fn queue_event(&mut self, event: Event) {
        self.queued_events.push(event);
    }

    /// Queues a message in the [`State`] for processing during an [`update`].
    ///
    /// [`update`]: Self::update
    pub fn queue_message(&mut self, message: P::Message) {
        self.queued_messages.push(message);
    }

    pub fn tick(&mut self, ctx: &mut Context) {
        self.program.tick(ctx);
    }

    /// Processes all the queued events and messages, rebuilding and redrawing
    /// the widgets of the linked [`Program`] if necessary.
    ///
    /// Returns the [`Command`] obtained from [`Program`] after updating it,
    /// only if an update was necessary.
    pub fn update(&mut self, ctx: &mut Context) -> Option<Vec<Task<P::Message>>> {
        let bounds = ctx.iced_manager.viewport.logical_size();
        let cursor_position = iced_conversion::cursor_position(
            ctx.window_state.cursor_physical_position,
            ctx.iced_manager.viewport.scale_factor(),
        );

        let mut user_interface = UserInterface::build(
            self.program.view(ctx),
            bounds,
            self.cache.take().unwrap(),
            &mut ctx.iced_manager.renderer,
        );

        let mut messages = Vec::new();

        user_interface.update(
            &self.queued_events,
            mouse::Cursor::Available(cursor_position),
            &mut ctx.iced_manager.renderer,
            &mut DummyClipboard {},
            &mut messages,
        );

        messages.append(&mut self.queued_messages);
        self.queued_events.clear();

        user_interface.draw(
            &mut ctx.iced_manager.renderer,
            &Theme::Dark,
            &iced_core::renderer::Style {
                text_color: Color::WHITE,
            },
            mouse::Cursor::Available(cursor_position),
        );

        self.cache = Some(user_interface.into_cache());

        if messages.is_empty() {
            None
        } else {
            let commands = messages
                .into_iter()
                .map(|message| self.program.update(ctx, message))
                .collect();
            Some(commands)
        }
    }
}
