use std::time::Instant;

use neothesia_core::render::{QuadInstance, QuadPipeline};
use wgpu_jumpstart::Color;

use crate::context::Context;
use crate::scene::playing_scene::animation::{Animated, Easing};
use crate::scene::playing_scene::LAYER_FG;

use super::Msg;

fn minus_icon() -> &'static str {
    "\u{F2EA}"
}

fn plus_icon() -> &'static str {
    "\u{F4FE}"
}

pub struct SpeedPill {
    plus: nuon::ElementId,
    minus: nuon::ElementId,

    plus_animation: Animated<bool, Instant>,
    minus_animation: Animated<bool, Instant>,
}

impl SpeedPill {
    pub fn new(elements: &mut nuon::ElementsMap<Msg>) -> Self {
        Self {
            plus: elements.insert(
                nuon::ElementBuilder::new()
                    .name("PlusButton")
                    .on_click(Msg::SpeedUpdateUp),
            ),
            minus: elements.insert(
                nuon::ElementBuilder::new()
                    .name("MinusButton")
                    .on_click(Msg::SpeedUpdateDown),
            ),

            plus_animation: Animated::new(false)
                .duration(50.0)
                .easing(Easing::EaseInOut)
                .delay(0.0),
            minus_animation: Animated::new(false)
                .duration(50.0)
                .easing(Easing::EaseInOut)
                .delay(0.0),
        }
    }

    pub fn update(
        &mut self,
        elements: &mut nuon::ElementsMap<Msg>,
        ctx: &mut Context,
        quad_pipeline: &mut QuadPipeline,
        y: f32,
        item: &nuon::RowItem,
        now: &Instant,
    ) {
        let text = &mut ctx.text_renderer;

        let y = y + 5.0;
        let w = item.width;
        let half_w = w / 2.0;

        let h = 19.0;

        elements.update(
            self.minus,
            nuon::Rect::new((item.x, y).into(), (half_w, h).into()),
        );
        elements.update(
            self.plus,
            nuon::Rect::new((item.x + half_w, y).into(), (half_w, h).into()),
        );

        for (element, animation, border_radius) in [
            (
                self.minus,
                &mut self.minus_animation,
                [10.0, 0.0, 10.0, 0.0],
            ),
            (self.plus, &mut self.plus_animation, [0.0, 10.0, 0.0, 10.0]),
        ] {
            if let Some(element) = elements.get(element) {
                animation.transition(element.hovered(), *now);

                let m = animation.animate(0.0, 20.0, *now) / 255.0;
                let c = 67.0 / 255.0;
                let color = Color::new(c + m, c + m, c + m, 1.0);

                quad_pipeline.push(
                    LAYER_FG,
                    QuadInstance {
                        position: element.rect().origin.into(),
                        size: element.rect().size.into(),
                        color: color.into_linear_rgba(),
                        border_radius,
                    },
                );
            }
        }

        let pad = 2.0;
        text.queue_icon(pad + item.x, y, h, minus_icon());
        text.queue_icon(item.x + item.width - h - pad, y, h, plus_icon());

        let label = format!("{}%", (ctx.config.speed_multiplier * 100.0).round());
        let buffer = text.gen_buffer_bold(13.0, &label);
        text.queue_buffer_centered(item.x, y, w, h, buffer);
    }
}
