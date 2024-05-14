use std::time::Duration;

use neothesia_core::{
    render::{QuadInstance, QuadPipeline, TextRenderer},
    utils::{Bbox, Point, Size},
};
use wgpu_jumpstart::Color;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
};

use crate::{context::Context, utils::window::WindowState, NeothesiaEvent};

use super::{
    animation::Animation, rewind_controller::RewindController, PlayingScene, EVENT_CAPTURED,
    EVENT_IGNORED,
};

mod button;
use button::Button;

#[derive(Default, Clone, Copy)]
enum Element {
    StartTick,
    EndTick,
    RepeatButton,
    BackButton,
    PlayButton,
    SettingsButton,
    #[default]
    None,
}

pub struct TopBar {
    bbox: Bbox<f32>,
    loop_tick_height: f32,

    animation: Animation,
    is_expanded: bool,

    settings_animation: Animation,
    drag: Element,
    hovered: Element,

    pub loop_start: Duration,
    pub loop_end: Duration,

    back_button: Button,
    play_button: Button,
    loop_button: Button,
    settings_button: Button,
    loop_start_tick: Bbox,
    loop_end_tick: Bbox,

    pub loop_active: bool,
    pub settings_active: bool,
}

macro_rules! color_u8 {
    ($r: expr, $g: expr, $b: expr, $a: expr) => {
        Color::new($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0, 1.0)
    };
}

const BAR_BG: Color = color_u8!(37, 35, 42, 1.0);
const BUTTON_HOVER: Color = color_u8!(47, 45, 52, 1.0);
const BLUE: Color = color_u8!(56, 145, 255, 1.0);
const LIGHT_MEASURE: Color = Color::new(1.0, 1.0, 1.0, 0.5);
const DARK_MEASURE: Color = Color::new(0.4, 0.4, 0.4, 1.0);
const LOOPER: Color = color_u8!(255, 56, 187, 1.0);
const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

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

impl TopBar {
    pub fn new() -> Self {
        Self {
            bbox: Bbox::new([0.0, 0.0], [0.0, 45.0 + 30.0]),
            loop_tick_height: 45.0 + 10.0,
            loop_button: Button::new(),
            back_button: Button::new(),
            play_button: Button::new(),
            settings_button: Button::new(),
            drag: Element::None,
            hovered: Element::None,
            loop_start: Duration::ZERO,
            loop_end: Duration::ZERO,
            loop_active: false,
            animation: Animation::new(),
            is_expanded: false,
            settings_animation: Animation::new(),
            loop_start_tick: Bbox::default(),
            loop_end_tick: Bbox::default(),
            settings_active: false,
        }
    }

    fn hovered(&self, x: f32, y: f32) -> Element {
        [
            (self.settings_button.bbox(), Element::SettingsButton),
            (self.loop_button.bbox(), Element::RepeatButton),
            (self.back_button.bbox(), Element::BackButton),
            (self.play_button.bbox(), Element::PlayButton),
            (&self.loop_start_tick, Element::StartTick),
            (&self.loop_end_tick, Element::EndTick),
        ]
        .into_iter()
        .find(|(e, _)| e.contains(x, y))
        .map(|e| e.1)
        .unwrap_or(Element::None)
    }

    pub fn handle_window_event(
        scene: &mut PlayingScene,
        ctx: &mut Context,
        event: &WindowEvent,
    ) -> bool {
        match &event {
            WindowEvent::MouseInput { state, button, .. } => {
                return Self::handle_mouse_input(scene, ctx, state, button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                Self::handle_cursor_moved(scene, ctx, position);
            }
            _ => {}
        }

        EVENT_IGNORED
    }

    fn handle_mouse_input(
        scene: &mut PlayingScene,
        ctx: &mut Context,
        state: &ElementState,
        button: &MouseButton,
    ) -> bool {
        match (state, button) {
            (ElementState::Pressed, MouseButton::Left) => match scene.top_bar.hovered {
                Element::RepeatButton => {
                    Self::toggle_loop(scene);
                    return EVENT_CAPTURED;
                }
                Element::PlayButton => {
                    scene.player.pause_resume();
                    return EVENT_CAPTURED;
                }
                Element::SettingsButton => {
                    scene.top_bar.settings_active = !scene.top_bar.settings_active;
                }
                Element::BackButton => {
                    ctx.proxy.send_event(NeothesiaEvent::MainMenu).ok();
                    return EVENT_CAPTURED;
                }
                Element::StartTick | Element::EndTick => {
                    scene.top_bar.drag = scene.top_bar.hovered;
                    return EVENT_CAPTURED;
                }
                _ => {
                    let pos = &ctx.window_state.cursor_logical_position;

                    if pos.y > 30.0
                        && pos.y < scene.top_bar.bbox.h()
                        && !scene.rewind_controller.is_rewinding()
                    {
                        scene
                            .rewind_controller
                            .start_mouse_rewind(&mut scene.player);

                        let x = ctx.window_state.cursor_logical_position.x;
                        let w = ctx.window_state.logical_size.width;

                        let p = x / w;
                        log::debug!("Progressbar: x:{},p:{}", x, p);
                        scene.player.set_percentage_time(p);
                        scene.keyboard.reset_notes();
                        return EVENT_CAPTURED;
                    }
                }
            },
            (ElementState::Released, MouseButton::Left) => {
                scene.top_bar.drag = Element::None;

                if let RewindController::Mouse { .. } = scene.rewind_controller {
                    scene.rewind_controller.stop_rewind(&mut scene.player);
                }
            }
            _ => {}
        }

        EVENT_IGNORED
    }

    fn handle_cursor_moved(
        scene: &mut PlayingScene,
        ctx: &mut Context,
        position: &PhysicalPosition<f64>,
    ) {
        let x = position.to_logical::<f32>(ctx.window_state.scale_factor).x;
        let y = position.to_logical::<f32>(ctx.window_state.scale_factor).y;
        let w = ctx.window_state.logical_size.width;

        scene.top_bar.hovered = scene.top_bar.hovered(x, y);

        match scene.top_bar.drag {
            Element::StartTick => {
                scene.top_bar.loop_start = scene.player.percentage_to_time(x / w);
            }
            Element::EndTick => {
                scene.top_bar.loop_end = scene.player.percentage_to_time(x / w);
            }
            Element::RepeatButton => {}
            _ => {}
        }
    }

    fn toggle_loop(scene: &mut PlayingScene) {
        scene.top_bar.loop_active = !scene.top_bar.loop_active;
        if scene.top_bar.loop_active {
            scene.top_bar.loop_start = scene.player.time();
            scene.top_bar.loop_end = scene.top_bar.loop_start + Duration::from_secs(3);
        }
    }

    pub fn update(scene: &mut PlayingScene, window_state: &WindowState, text: &mut TextRenderer) {
        let PlayingScene {
            top_bar,
            fg_quad_pipeline,
            ref player,
            ref rewind_controller,
            ..
        } = scene;

        top_bar.bbox.size.w = window_state.logical_size.width;

        let h = top_bar.bbox.h();
        let is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        top_bar.is_expanded = is_hovered;
        top_bar.is_expanded |= top_bar.settings_active;
        top_bar.is_expanded |= matches!(rewind_controller, RewindController::Mouse { .. });

        top_bar.animation.update(top_bar.is_expanded);
        top_bar.settings_animation.update(top_bar.settings_active);

        if !top_bar.is_expanded {
            let progress_x = top_bar.bbox.w() * player.percentage();
            draw_rect(
                fg_quad_pipeline,
                &Bbox::new([0.0, 0.0], [progress_x, 5.0]),
                &BLUE,
            );
        }

        if top_bar.animation.is_done() {
            return;
        }

        let bar_animation = top_bar.animation.expo_out(top_bar.is_expanded);

        top_bar.bbox.pos.y = -h + (bar_animation * h);

        draw_rect(fg_quad_pipeline, &top_bar.bbox, &BAR_BG);

        for f in [
            update_proggress_bar,
            update_buttons,
            update_looper,
            update_settings_card,
        ] {
            f(scene, text);
        }
    }
}

fn update_proggress_bar(scene: &mut PlayingScene, _text: &mut TextRenderer) {
    let PlayingScene {
        top_bar,
        fg_quad_pipeline,
        player,
        ..
    } = scene;

    let y = top_bar.bbox.y() + 30.0;
    let h = top_bar.bbox.h() - 30.0;
    let w = top_bar.bbox.w();

    let progress_x = w * player.percentage();
    draw_rect(
        fg_quad_pipeline,
        &Bbox::new([0.0, y], [progress_x, h]),
        &BLUE,
    );

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

        draw_rect(fg_quad_pipeline, &Bbox::new([x, y], [1.0, h]), &color);
    }
}

fn update_buttons(scene: &mut PlayingScene, text: &mut TextRenderer) {
    let PlayingScene {
        top_bar,
        fg_quad_pipeline,
        ..
    } = scene;

    let y = top_bar.bbox.y();
    let w = top_bar.bbox.w();

    top_bar
        .back_button
        .set_pos((0.0, y))
        .set_hovered(matches!(top_bar.hovered, Element::BackButton))
        .set_icon(left_arrow_icon())
        .draw(fg_quad_pipeline, text);

    let mut x = w;

    x -= 30.0;
    top_bar
        .settings_button
        .set_pos((x, y))
        .set_hovered(matches!(top_bar.hovered, Element::SettingsButton))
        .set_icon(if top_bar.settings_active {
            gear_fill_icon()
        } else {
            gear_icon()
        })
        .draw(fg_quad_pipeline, text);

    x -= 30.0;
    top_bar
        .loop_button
        .set_pos((x, y))
        .set_hovered(matches!(top_bar.hovered, Element::RepeatButton))
        .set_icon(repeat_icon())
        .draw(fg_quad_pipeline, text);

    x -= 30.0;
    top_bar
        .play_button
        .set_pos((x, y))
        .set_hovered(matches!(top_bar.hovered, Element::PlayButton))
        .set_icon(if scene.player.is_paused() {
            play_icon()
        } else {
            pause_icon()
        })
        .draw(fg_quad_pipeline, text);
}

fn update_looper(scene: &mut PlayingScene, _text: &mut TextRenderer) {
    let PlayingScene {
        top_bar,
        fg_quad_pipeline,
        ref player,
        ..
    } = scene;

    if !top_bar.loop_active {
        return;
    }

    let animation = top_bar.animation.expo_out(top_bar.is_expanded);

    let h = top_bar.loop_tick_height;
    let w = top_bar.bbox.w();
    let offset = 30.0 + h;
    let y = 30.0 - offset + (animation * offset);

    let tick_size = Size::new(5.0, h);
    let tick_pos = |start: &Duration| Point {
        x: player.time_to_percentage(start) * w,
        y,
    };

    top_bar.loop_start_tick = Bbox::new(tick_pos(&top_bar.loop_start), tick_size);
    top_bar.loop_end_tick = Bbox::new(tick_pos(&top_bar.loop_end), tick_size);

    let (start_color, end_color) = match (top_bar.hovered, top_bar.drag) {
        (Element::StartTick, _) | (_, Element::StartTick) => (WHITE, LOOPER),
        (Element::EndTick, _) | (_, Element::EndTick) => (LOOPER, WHITE),
        _ => (LOOPER, LOOPER),
    };

    let color = Color { a: 0.35, ..LOOPER };

    let length = top_bar.loop_end_tick.x() - top_bar.loop_start_tick.x();

    let mut bg_box = top_bar.loop_start_tick;
    bg_box.size.w = length;

    draw_rect(fg_quad_pipeline, &bg_box, &color);
    draw_rect(fg_quad_pipeline, &top_bar.loop_start_tick, &start_color);
    draw_rect(fg_quad_pipeline, &top_bar.loop_end_tick, &end_color);
}

fn update_settings_card(scene: &mut PlayingScene, _text: &mut TextRenderer) {
    let PlayingScene {
        top_bar,
        fg_quad_pipeline,
        ..
    } = scene;

    let settings_animation = top_bar.settings_animation.expo_out(top_bar.settings_active);

    let y = top_bar.bbox.y();
    let h = top_bar.bbox.h();
    let w = top_bar.bbox.w();

    if !top_bar.settings_animation.is_done() {
        let card_w = 300.0;
        let card_x = card_w - (settings_animation * card_w);

        fg_quad_pipeline.push(QuadInstance {
            position: [card_x + w - card_w, y + h + 1.0],
            size: [card_w, 100.0],
            color: BAR_BG.into_linear_rgba(),
            border_radius: [10.0, 0.0, 10.0, 0.0],
        });
    }
}

fn draw_rect(quad_pipeline: &mut QuadPipeline, bbox: &Bbox, color: &Color) {
    quad_pipeline.push(QuadInstance {
        position: bbox.pos.into(),
        size: bbox.size.into(),
        color: color.into_linear_rgba(),
        ..Default::default()
    });
}
