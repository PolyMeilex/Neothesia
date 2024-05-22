use super::Element;
use iced_core::{image::Handle as ImageHandle, Alignment};

mod theme;

pub struct StatsContainer<'a, MSG> {
    image: Option<ImageHandle>,
    date: String,
    place: String,
    score: String,
    notes_hits: String,
    notes_missed: String,
    wrong_notes: String,
    correct_notes_duration: String,
    body: Option<Element<'a, MSG>>,

    header: bool,
}

impl<'a, MSG: 'a + Clone> Default for StatsContainer<'a, MSG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, MSG: 'a + Clone> StatsContainer<'a, MSG> {
    pub fn new() -> Self {
        Self {
            image: None,
            date: String::new(),
            place: String::new(),
            score: String::new(),
            notes_hits: String::new(),
            notes_missed: String::new(),
            wrong_notes: String::new(),
            correct_notes_duration: String::new(),
            body: None,

            header: false,
        }
    }

    pub fn image(mut self, image: u32) -> Self {
        self.image = match image {
            0 => Some(ImageHandle::from_memory(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/stats_container/img/trophy_placeholder.png"
                ))
                .to_vec(),
            )),
            1 => Some(ImageHandle::from_memory(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/stats_container/img/first_place.png"
                ))
                .to_vec(),
            )),
            2 => Some(ImageHandle::from_memory(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/stats_container/img/second_place.png"
                ))
                .to_vec(),
            )),
            3 => Some(ImageHandle::from_memory(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/stats_container/img/third_place.png"
                ))
                .to_vec(),
            )),
            _ => None,
        };
        self
    }

    pub fn date(mut self, date: impl ToString) -> Self {
        self.date = date.to_string();
        self
    }

    pub fn place(mut self, place: impl ToString) -> Self {
        self.place = place.to_string();
        self
    }

    pub fn score(mut self, score: impl ToString) -> Self {
        self.score = score.to_string();
        self
    }

    pub fn notes_hits(mut self, hits: impl ToString) -> Self {
        self.notes_hits = hits.to_string();
        self
    }

    pub fn notes_missed(mut self, missed: impl ToString) -> Self {
        self.notes_missed = missed.to_string();
        self
    }

    pub fn wrong_notes(mut self, wrong: impl ToString) -> Self {
        self.wrong_notes = wrong.to_string();
        self
    }

    pub fn correct_notes_duration(mut self, duration: impl ToString) -> Self {
        self.correct_notes_duration = duration.to_string();
        self
    }

    pub fn body(mut self, body: impl Into<Element<'a, MSG>>) -> Self {
        self.body = Some(body.into());
        self
    }
    pub fn header(mut self, header: bool) -> Self {
        self.header = header;
        self
    }
}

impl<'a, M: Clone + 'a> From<StatsContainer<'a, M>> for Vec<Element<'a, M>> {
    fn from(card: StatsContainer<'a, M>) -> Self {
        let columns = vec![
            (card.place, 90),
            (card.date, 190),
            (card.score, 90),
            (card.notes_hits, 90),
            (card.notes_missed, 100),
            (card.wrong_notes, 90),
            (card.correct_notes_duration, 120),
        ];

        let header_row = columns
            .iter()
            .map(|(text, width)| {
                let container =
                    iced_widget::container(iced_widget::text(text.clone()).size(12)).width(*width);

                // Set background color

                container.into()
            })
            .collect::<Vec<_>>();

        let header = iced_widget::row(header_row)
            .spacing(0)
            .align_items(Alignment::Start);

        let image_container = if let Some(image) = card.image {
            iced_widget::container(iced_widget::image(image).width(40)).into()
        } else {
            iced_widget::container(iced_widget::text(""))
                .width(0)
                .into()
        };

        let text_container =
            iced_widget::container(iced_widget::column(vec![header.into()]).spacing(6));

        let centered_container = iced_widget::container(
            iced_widget::row(vec![image_container, text_container.padding(10).into()])
                .align_items(Alignment::Center),
        );

        let mut children = vec![];

        if card.header {
            let centered_with_style = iced_widget::container(centered_container)
                .padding(10)
                .style(theme::card())
                .into();
            children.push(centered_with_style);
        } else {
            children.push(centered_container.padding(8).into());
        }

        children
    }
}
