use iced_core::{Alignment, Length};
use iced_style::Theme;
use iced_widget::{column as col, row, Component};
use neothesia_iced_widgets::{Element, NeoBtn, Renderer};

use crate::context::Context;

use super::{center_x, centered_text};

pub struct ExitPage<'a, MSG> {
    _ctx: &'a Context,
    on_back: Option<Box<dyn Fn() -> MSG>>,
    on_exit: Option<Box<dyn Fn() -> MSG>>,
}

impl<'a, MSG> ExitPage<'a, MSG> {
    pub fn new(ctx: &'a Context) -> Self {
        Self {
            _ctx: ctx,
            on_back: None,
            on_exit: None,
        }
    }

    pub fn on_back(mut self, cb: impl Fn() -> MSG + 'static) -> Self {
        self.on_back = Some(Box::new(cb));
        self
    }

    pub fn on_exit(mut self, cb: impl Fn() -> MSG + 'static) -> Self {
        self.on_exit = Some(Box::new(cb));
        self
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Cancel,
    Exit,
}

impl<'a, MSG> Component<MSG, Theme, Renderer> for ExitPage<'a, MSG> {
    type State = ();
    type Event = Event;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<MSG> {
        match event {
            Event::Cancel => self.on_back.as_ref().map(|cb| cb()),
            Event::Exit => self.on_exit.as_ref().map(|cb| cb()),
        }
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event> {
        let output = centered_text("Do you want to exit?").size(30);

        let select_row = row![
            NeoBtn::new_with_label("No")
                .width(Length::Fill)
                .on_press(Event::Cancel),
            NeoBtn::new_with_label("Yes")
                .width(Length::Fill)
                .on_press(Event::Exit),
        ]
        .spacing(5)
        .height(Length::Fixed(50.0));

        let controls = col![output, select_row]
            .align_items(Alignment::Center)
            .width(Length::Fixed(650.0))
            .spacing(30);

        center_x(controls).center_y().into()
    }
}

impl<'a, MSG> From<ExitPage<'a, MSG>> for Element<'a, MSG>
where
    MSG: 'a,
{
    fn from(page: ExitPage<'a, MSG>) -> Self {
        iced_widget::component(page)
    }
}
