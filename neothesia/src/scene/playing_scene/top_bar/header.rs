use std::time::Instant;

use speed_pill::SpeedPill;

use crate::{context::Context, scene::playing_scene::PlayingScene};

use super::{looper::LooperMsg, Button, Msg};

mod speed_pill;

fn gear_icon() -> &'static str {
    "\u{F3E5}"
}

fn gear_fill_icon() -> &'static str {
    "\u{F3E2}"
}

fn repeat_icon() -> &'static str {
    "\u{f130}"
}

fn play_icon() -> &'static str {
    "\u{f4f4}"
}

fn pause_icon() -> &'static str {
    "\u{f4c3}"
}

fn left_arrow_icon() -> &'static str {
    "\u{f12f}"
}

pub struct Header {
    layout: nuon::TriRowLayout,
    back_button: Button,
    play_button: Button,
    loop_button: Button,
    settings_button: Button,

    speed_pill: SpeedPill,
}

impl Header {
    pub fn new(elements: &mut nuon::ElementsMap<Msg>) -> Self {
        let mut back_button = Button::new(
            elements,
            nuon::ElementBuilder::new()
                .name("BackButton")
                .on_click(Msg::GoBack),
        );
        let mut play_button = Button::new(
            elements,
            nuon::ElementBuilder::new()
                .name("PlayButton")
                .on_click(Msg::PlayResume),
        );
        let mut loop_button = Button::new(
            elements,
            nuon::ElementBuilder::new()
                .name("LoopButton")
                .on_click(Msg::LooperEvent(LooperMsg::Toggle)),
        );
        let mut settings_button = Button::new(
            elements,
            nuon::ElementBuilder::new()
                .name("SettingsButton")
                .on_click(Msg::SettingsToggle),
        );

        back_button.set_icon(left_arrow_icon());
        play_button.set_icon(pause_icon());
        loop_button.set_icon(repeat_icon());
        settings_button.set_icon(gear_icon());

        Self {
            layout: nuon::TriRowLayout::new(),
            back_button,
            play_button,
            loop_button,
            settings_button,

            speed_pill: SpeedPill::new(elements),
        }
    }

    pub fn invalidate_layout(&mut self) {
        self.layout.invalidate();
    }

    fn update_button_icons(scene: &mut PlayingScene) {
        let PlayingScene {
            top_bar, player, ..
        } = scene;

        top_bar.header.play_button.set_icon(if player.is_paused() {
            play_icon()
        } else {
            pause_icon()
        });

        top_bar
            .header
            .settings_button
            .set_icon(if top_bar.settings_active {
                gear_fill_icon()
            } else {
                gear_icon()
            });
    }

    pub fn update(scene: &mut PlayingScene, ctx: &mut Context, now: &Instant) {
        Self::update_button_icons(scene);

        let PlayingScene {
            top_bar,
            quad_pipeline,
            ..
        } = scene;

        let y = top_bar.bbox.origin.y;
        let w = top_bar.bbox.size.width;

        let (_back_id,) = top_bar.header.layout.start.once(|row| {
            (
                row.push(30.0),
                //
            )
        });

        let (center_box,) = top_bar.header.layout.center.once(|row| {
            (
                row.push(90.0),
                //
            )
        });

        let (_play, _loop, _settings) = top_bar.header.layout.end.once(|row| {
            (
                row.push(30.0),
                row.push(30.0),
                row.push(30.0),
                //
            )
        });

        top_bar.header.layout.resolve(0.0, w);

        top_bar.header.speed_pill.update(
            &mut top_bar.elements,
            ctx,
            quad_pipeline,
            y,
            &top_bar.header.layout.center.items()[center_box],
            now,
        );

        let start_row = top_bar.header.layout.start.items();
        let _center_row = top_bar.header.layout.center.items();
        let end_row = top_bar.header.layout.end.items();

        for (btn, item) in [&mut top_bar.header.back_button].into_iter().zip(start_row) {
            btn.update(
                &mut top_bar.elements,
                nuon::Rect::new((item.x, y).into(), (item.width, 30.0).into()),
            );
        }

        for (btn, item) in [
            &mut top_bar.header.play_button,
            &mut top_bar.header.loop_button,
            &mut top_bar.header.settings_button,
        ]
        .into_iter()
        .zip(end_row)
        {
            btn.update(
                &mut top_bar.elements,
                nuon::Rect::new((item.x, y).into(), (item.width, 30.0).into()),
            );
        }

        for btn in [
            &top_bar.header.back_button,
            &top_bar.header.play_button,
            &top_bar.header.loop_button,
            &top_bar.header.settings_button,
        ] {
            btn.draw(quad_pipeline, &mut ctx.text_renderer);
        }
    }
}
