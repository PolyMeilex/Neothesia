use crate::iced_utils::iced_state::Element;
use iced_core::{Alignment, Length, Renderer};
use iced_style::Theme;
use iced_widget::{column as col, row};

pub struct Layout<'a, Message> {
    top: Option<Element<'a, Message>>,
    body: Option<Element<'a, Message>>,
    bottom: Option<Element<'a, Message>>,
}

impl<'a, M: 'static> Default for Layout<'a, M> {
    fn default() -> Self {
        Self {
            top: None,
            body: None,
            bottom: None,
        }
    }
}

impl<'a, M: 'static> Layout<'a, M> {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn top(mut self, top: impl Into<Element<'a, M>>) -> Self {
        self.top = Some(top.into());
        self
    }

    pub fn body(mut self, body: impl Into<Element<'a, M>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn bottom(mut self, bottom: impl Into<Element<'a, M>>) -> Self {
        self.bottom = Some(bottom.into());
        self
    }

    pub fn build(self) -> Element<'a, M> {
        let body = col![].push_if(self.body);
        let body = col![body]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center);

        let top = self
            .top
            .map(|top| col![top].width(Length::Fill).align_items(Alignment::Center));
        let bottom = self.bottom.map(|bottom| {
            col![bottom]
                .width(Length::Fill)
                .align_items(Alignment::Center)
        });

        col![]
            .push_if(top)
            .push(body)
            .push_if(bottom)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<'a, M: 'static> From<Layout<'a, M>> for Element<'a, M> {
    fn from(val: Layout<'a, M>) -> Self {
        val.build()
    }
}

pub struct BarLayout<'a, Message> {
    left: Option<Element<'a, Message>>,
    center: Option<Element<'a, Message>>,
    right: Option<Element<'a, Message>>,
}

impl<'a, M: 'static> Default for BarLayout<'a, M> {
    fn default() -> Self {
        Self {
            left: None,
            center: None,
            right: None,
        }
    }
}

impl<'a, M: 'static> BarLayout<'a, M> {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn left(mut self, left: impl Into<Element<'a, M>>) -> Self {
        self.left = Some(left.into());
        self
    }

    pub fn center(mut self, center: impl Into<Element<'a, M>>) -> Self {
        self.center = Some(center.into());
        self
    }

    pub fn right(mut self, right: impl Into<Element<'a, M>>) -> Self {
        self.right = Some(right.into());
        self
    }

    pub fn build(self) -> Element<'a, M> {
        let left = row![].push_if(self.left).width(Length::Fill);
        let center = row![].push_if(self.center).width(Length::Fill);
        let right = row![].push_if(self.right).width(Length::Fill);

        row![left, center, right]
            .align_items(Alignment::Center)
            .into()
    }
}

impl<'a, M: 'static> From<BarLayout<'a, M>> for Element<'a, M> {
    fn from(val: BarLayout<'a, M>) -> Self {
        val.build()
    }
}

pub trait PushIf<'a, M, R> {
    fn push_if(self, item: Option<impl Into<iced_core::Element<'a, M, Theme, R>>>) -> Self;
}

impl<'a, M, R: Renderer> PushIf<'a, M, R> for iced_widget::Row<'a, M, Theme, R> {
    fn push_if(self, item: Option<impl Into<iced_core::Element<'a, M, Theme, R>>>) -> Self {
        if let Some(item) = item {
            self.push(item)
        } else {
            self
        }
    }
}

impl<'a, M, R: Renderer> PushIf<'a, M, R> for iced_widget::Column<'a, M, Theme, R> {
    fn push_if(self, item: Option<impl Into<iced_core::Element<'a, M, Theme, R>>>) -> Self {
        if let Some(item) = item {
            self.push(item)
        } else {
            self
        }
    }
}

impl<'a, M: 'a> PushIf<'a, M, super::Renderer>
    for super::preferences_group::PreferencesGroup<'a, M>
{
    fn push_if(
        self,
        item: Option<impl Into<iced_core::Element<'a, M, Theme, super::Renderer>>>,
    ) -> Self {
        if let Some(item) = item {
            self.push(item)
        } else {
            self
        }
    }
}
