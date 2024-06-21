use iced_core::{Font, Length, Theme};
use iced_widget::{column, container, row, text};

use super::{Element, Renderer};

mod theme;

#[derive(Default)]
struct PreferencesGroupHeader {
    title: Option<String>,
    subtitle: Option<String>,
}

pub struct PreferencesGroup<'a, MSG> {
    header: Option<PreferencesGroupHeader>,
    items: Vec<Element<'a, MSG>>,
}

impl<'a, MSG: 'a> Default for PreferencesGroup<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'a> PreferencesGroup<'a, MSG> {
    pub fn new() -> Self {
        Self {
            header: None,
            items: Vec::new(),
        }
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.header.get_or_insert_with(Default::default).title = Some(title.to_string());
        self
    }

    #[allow(unused)]
    pub fn subtitle(mut self, subtitle: impl ToString) -> Self {
        self.header.get_or_insert_with(Default::default).subtitle = Some(subtitle.to_string());
        self
    }

    pub fn push(mut self, item: impl Into<Element<'a, MSG>>) -> Self {
        self.items.push(item.into());
        self
    }

    pub fn push_maybe(self, child: Option<impl Into<Element<'a, MSG>>>) -> Self {
        if let Some(child) = child {
            self.push(child)
        } else {
            self
        }
    }

    pub fn build(self) -> Element<'a, MSG> {
        let header = self.header.map(|header| group_header(header));
        let body = group_body(self.items);

        column![].push_maybe(header).push(body).spacing(14).into()
    }
}

fn group_body<'a, M: 'a>(items: Vec<Element<'a, M>>) -> Element<'a, M> {
    let mut needs_sep = false;
    let mut items = items.into_iter().peekable();

    let mut coll = column![];

    loop {
        if needs_sep && items.peek().is_some() {
            needs_sep = false;

            let separator = container(row![])
                .width(Length::Fill)
                .height(1)
                .style(theme::separator);
            coll = coll.push(separator);
        } else {
            needs_sep = true;
            if let Some(item) = items.next() {
                coll = coll.push(item);
            } else {
                break;
            }
        }
    }

    container(coll).style(theme::card).into()
}

fn triple_split<'a, T: 'a>(
    prefix: Option<Element<'a, T>>,
    center: Option<Element<'a, T>>,
    suffix: Option<Element<'a, T>>,
) -> iced_widget::Row<'a, T, Theme, Renderer> {
    let mut row = row![];

    row = row.push(row![].push_maybe(prefix).width(Length::Shrink));
    row = row.push(row![].push_maybe(center).width(Length::Fill));
    row = row.push(row![].push_maybe(suffix).width(Length::Shrink));

    row.align_items(iced_core::Alignment::Center).spacing(6)
}

fn group_header<'a, T: 'a>(data: PreferencesGroupHeader) -> Element<'a, T> {
    let title = data.title.map(|title| {
        text(title)
            .font(Font {
                weight: iced_core::font::Weight::Semibold,
                ..Font::DEFAULT
            })
            .size(14.6)
    });
    let subtitle = data
        .subtitle
        .map(|title| text(title).style(theme::subtitle).size(12.2));

    let header = column![].push_maybe(title).push_maybe(subtitle);

    triple_split(None, Some(header.into()), None).into()
}

fn title<'a, T: 'a>(
    title: Option<String>,
    subtitle: Option<String>,
) -> iced_widget::Column<'a, T, Theme, Renderer> {
    column![]
        .push_maybe(title.map(|title| text(title).size(14.6)))
        .push_maybe(subtitle.map(|subtitle| text(subtitle).size(12.2).style(theme::subtitle)))
}

pub struct ActionRow<'a, MSG> {
    prefix: Option<Element<'a, MSG>>,
    title: Option<String>,
    subtitle: Option<String>,
    suffix: Option<Element<'a, MSG>>,
}

impl<'a, MSG: 'a> Default for ActionRow<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'a> ActionRow<'a, MSG> {
    pub fn new() -> Self {
        Self {
            prefix: None,
            title: None,
            subtitle: None,
            suffix: None,
        }
    }

    #[allow(unused)]
    pub fn prefix(mut self, prefix: impl Into<Element<'a, MSG>>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn subtitle(mut self, subtitle: impl ToString) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn suffix(mut self, suffix: impl Into<Element<'a, MSG>>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    fn build(self) -> iced_widget::Row<'a, MSG, Theme, Renderer> {
        let center = if self.title.is_some() || self.subtitle.is_some() {
            Some(title(self.title, self.subtitle).into())
        } else {
            None
        };

        triple_split(self.prefix, center, self.suffix)
            .width(700)
            .padding(15)
    }
}

impl<'a, M: 'a> From<ActionRow<'a, M>> for Element<'a, M> {
    fn from(val: ActionRow<'a, M>) -> Self {
        val.build().into()
    }
}
