use std::time::Instant;

use crate::{
    context::Context,
    scene::playing_scene::{rewind_controller::RewindController, PlayingScene},
};

use super::{draw_rect, Msg, BLUE, DARK_MEASURE, LIGHT_MEASURE};

#[derive(Debug, Clone)]
pub enum ProgressBarMsg {
    Pressed,
    Released,
}

pub struct ProgressBar {
    id: nuon::ElementId,
}

impl ProgressBar {
    pub fn new(elements: &mut nuon::ElementsMap<Msg>) -> Self {
        let id = elements.insert(
            nuon::ElementBuilder::new()
                .name("ProgressBar")
                .on_pressed(Msg::ProggresBar(ProgressBarMsg::Pressed))
                .on_release(Msg::ProggresBar(ProgressBarMsg::Released)),
        );

        Self { id }
    }

    pub fn on_msg(scene: &mut PlayingScene, ctx: &mut Context, msg: ProgressBarMsg) {
        let PlayingScene {
            player,
            keyboard,
            rewind_controller,
            ..
        } = scene;

        match msg {
            ProgressBarMsg::Pressed => {
                if !rewind_controller.is_rewinding() {
                    rewind_controller.start_mouse_rewind(player);

                    let x = ctx.window_state.cursor_logical_position.x;
                    let w = ctx.window_state.logical_size.width;

                    let p = x / w;
                    player.set_percentage_time(p);
                    keyboard.reset_notes();
                }
            }
            ProgressBarMsg::Released => {
                if let RewindController::Mouse { .. } = rewind_controller {
                    rewind_controller.stop_rewind(player);
                }
            }
        }
    }

    pub fn update(scene: &mut PlayingScene, _ctx: &mut Context, _now: &Instant) {
        let PlayingScene {
            top_bar,
            quad_pipeline,
            player,
            ..
        } = scene;

        let y = top_bar.bbox.origin.y + 30.0;
        let h = top_bar.bbox.size.height - 30.0;
        let w = top_bar.bbox.size.width;

        let progress_x = w * player.percentage();

        let mut rect = nuon::Rect::new((0.0, y).into(), (w, h).into());
        top_bar.elements.update(top_bar.progress_bar.id, rect);

        rect.size.width = progress_x;
        draw_rect(quad_pipeline, &rect, &BLUE);

        for m in player.song().file.measures.iter() {
            let length = player.length().as_secs_f32();
            let start = player.leed_in().as_secs_f32() / length;
            let measure = m.as_secs_f32() / length;

            let x = (start + measure) * w;

            let color = if x < progress_x {
                LIGHT_MEASURE
            } else {
                DARK_MEASURE
            };

            draw_rect(
                quad_pipeline,
                &nuon::Rect::new((x, y).into(), (1.0, h).into()),
                &color,
            );
        }
    }
}
