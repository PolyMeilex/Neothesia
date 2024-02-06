use neothesia_core::render::QuadInstance;
use wgpu_jumpstart::Color;

use crate::utils::window::WindowState;

use super::{animation, PlayingScene};

#[derive(Default)]
pub struct TopBar {
    animation: f32,
}

macro_rules! color_u8 {
    ($r: expr, $g: expr, $b: expr, $a: expr) => {
        Color::new($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0, 1.0)
    };
}

const BAR_BG: Color = color_u8!(37, 35, 42, 1.0);
const BLUE: Color = color_u8!(56, 145, 255, 1.0);
const LIGHT_MEASURE: Color = Color::new(1.0, 1.0, 1.0, 0.5);
const DARK_MEASURE: Color = Color::new(0.4, 0.4, 0.4, 1.0);

impl TopBar {
    pub fn update(scene: &mut PlayingScene, window_state: &WindowState) {
        let top_bar = &mut scene.top_bar;
        let quad_pipeline = &mut scene.fg_quad_pipeline;
        let player = &scene.player;
        let rewind_controler = &scene.rewind_controler;

        let h = 45.0;
        let w = window_state.logical_size.width;
        let progress_x = w * player.percentage();

        let is_hovered =
            window_state.cursor_logical_position.y < h || rewind_controler.is_rewinding();

        if !is_hovered {
            quad_pipeline.instances().push(QuadInstance {
                position: [0.0, 0.0],
                size: [progress_x, 5.0],
                color: BLUE.into_linear_rgba(),
                ..Default::default()
            });
        }

        if is_hovered {
            top_bar.animation += 0.04;
        } else {
            top_bar.animation -= 0.1;
        }

        top_bar.animation = top_bar.animation.min(1.0);
        top_bar.animation = top_bar.animation.max(0.0);

        if top_bar.animation == 0.0 {
            return;
        }

        let bar_animation = if is_hovered {
            animation::expo_out(top_bar.animation)
        } else {
            top_bar.animation
        };

        let y = -h + (bar_animation * h);

        quad_pipeline.instances().push(QuadInstance {
            position: [0.0, y],
            size: [w, h],
            color: BAR_BG.into_linear_rgba(),
            ..Default::default()
        });

        let progress_x = w * player.percentage();
        quad_pipeline.instances().push(QuadInstance {
            position: [0.0, y],
            size: [progress_x, h],
            color: BLUE.into_linear_rgba(),
            ..Default::default()
        });

        for m in player.song().file.measures.iter() {
            let lenght = player.lenght().as_secs_f32();
            let start = player.leed_in().as_secs_f32() / lenght;
            let measure = m.as_secs_f32() / lenght;

            let x = (start + measure) * w;

            let color = if x < progress_x {
                LIGHT_MEASURE
            } else {
                DARK_MEASURE
            };

            quad_pipeline.instances().push(QuadInstance {
                position: [x, y],
                size: [1.0, h],
                color: color.into_linear_rgba(),
                ..Default::default()
            });
        }
    }
}
