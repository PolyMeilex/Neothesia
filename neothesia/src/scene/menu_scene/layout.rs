use crate::iced_utils::iced_state::Element;
use iced_core::{Alignment, Length};
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
        let mut root = col![].width(Length::Fill).height(Length::Fill);

        if let Some(top) = self.top {
            let top = col![top].width(Length::Fill).align_items(Alignment::Center);
            root = root.push(top);
        }

        let body = if let Some(body) = self.body {
            col![body]
        } else {
            col![]
        };

        let body = col![body]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center);
        root = root.push(body);

        if let Some(bottom) = self.bottom {
            let bottom = col![bottom]
                .width(Length::Fill)
                .align_items(Alignment::Center);
            root = root.push(bottom);
        }

        root.into()
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
        let mut left = row![].width(Length::Fill);
        let mut center = row![].width(Length::Fill);
        let mut right = row![].width(Length::Fill);

        if let Some(item) = self.left {
            left = left.push(item);
        }
        if let Some(item) = self.center {
            center = center.push(item);
        }
        if let Some(item) = self.right {
            right = right.push(item);
        }

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
