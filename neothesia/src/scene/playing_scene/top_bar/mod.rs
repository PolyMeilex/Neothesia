use std::time::Instant;

use neothesia_core::{
    render::{QuadInstance, QuadPipeline},
    utils::Rect,
};
use wgpu_jumpstart::Color;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
};

use crate::{context::Context, NeothesiaEvent};

use super::{
    animation::{Animated, Easing},
    rewind_controller::RewindController,
    PlayingScene, EVENT_CAPTURED, EVENT_IGNORED, LAYER_FG,
};

mod button;
use button::Button;

mod header;
use header::Header;

mod progress_bar;
use progress_bar::{ProgressBar, ProgressBarMsg};

mod looper;
use looper::{Looper, LooperMsg};

#[derive(Debug, Clone)]
enum Msg {
    GoBack,
    PlayResume,
    SettingsToggle,
    ProggresBar(ProgressBarMsg),
    LooperEvent(LooperMsg),
    SpeedUpdateUp,
    SpeedUpdateDown,
}

pub struct TopBar {
    elements: nuon::ElementsMap<Msg>,

    bbox: Rect<f32>,
    loop_tick_height: f32,

    animation: Animated<bool, Instant>,
    is_expanded: bool,

    settings_animation: Animated<bool, Instant>,

    header: Header,
    progress_bar: ProgressBar,
    pub looper: Looper,

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

impl TopBar {
    pub fn new() -> Self {
        let mut elements = nuon::ElementsMap::new();

        let header = Header::new(&mut elements);

        let progress_bar = ProgressBar::new(&mut elements);

        let looper = Looper::new(&mut elements);
        let bbox = Rect::new((0.0, 0.0).into(), (0.0, 45.0 + 30.0).into());

        Self {
            elements,

            bbox,
            loop_tick_height: 45.0 + 10.0,

            header,

            animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),
            settings_animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),

            is_expanded: false,
            progress_bar,
            looper,
            settings_active: false,
        }
    }

    fn on_msg(scene: &mut PlayingScene, ctx: &mut Context, msg: Msg) {
        match msg {
            Msg::GoBack => {
                ctx.proxy
                    .send_event(NeothesiaEvent::MainMenu(Some(scene.player.song().clone())))
                    .ok();
            }
            Msg::PlayResume => {
                scene.player.pause_resume();
            }
            Msg::SettingsToggle => {
                scene.top_bar.settings_active = !scene.top_bar.settings_active;
            }
            Msg::SpeedUpdateUp => {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() + 0.1);
            }
            Msg::SpeedUpdateDown => {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() - 0.1);
            }
            Msg::LooperEvent(msg) => {
                Looper::on_msg(scene, ctx, msg);
            }
            Msg::ProggresBar(msg) => {
                ProgressBar::on_msg(scene, ctx, msg);
            }
        }
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
            WindowEvent::Resized(_) => {
                scene.top_bar.header.invalidate_layout();
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
            (ElementState::Pressed, MouseButton::Left) => {
                if let Some(msg) = scene
                    .top_bar
                    .elements
                    .on_press()
                    .and_then(|e| e.on_pressed())
                    .cloned()
                {
                    Self::on_msg(scene, ctx, msg);
                    return EVENT_CAPTURED;
                }
            }
            (ElementState::Released, MouseButton::Left) => {
                let hovered_element = scene.top_bar.elements.hovered_element_id();
                if let Some(element) = scene.top_bar.elements.on_release() {
                    let click_event = element
                        .on_click()
                        // Is hover still over the element that started the press grab
                        .filter(|_| hovered_element == Some(element.id()))
                        .cloned();

                    let release_event = element.on_release().cloned();

                    if let Some(msg) = click_event {
                        Self::on_msg(scene, ctx, msg);
                    }
                    if let Some(msg) = release_event {
                        Self::on_msg(scene, ctx, msg);
                    }

                    return EVENT_CAPTURED;
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

        scene.top_bar.elements.update_cursor_pos((x, y).into());

        if let Some(msg) = scene
            .top_bar
            .elements
            .current_mouse_grab()
            .and_then(|e| e.on_cursor_move())
            .cloned()
        {
            Self::on_msg(scene, ctx, msg);
        }
    }

    #[profiling::function]
    pub fn update(scene: &mut PlayingScene, ctx: &mut Context) {
        let PlayingScene {
            top_bar,
            quad_pipeline,
            ref player,
            ref rewind_controller,
            ..
        } = scene;

        let window_state = &ctx.window_state;

        top_bar.bbox.size.width = window_state.logical_size.width;

        let h = top_bar.bbox.size.height;
        let is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        top_bar.is_expanded = is_hovered;
        top_bar.is_expanded |= top_bar.settings_active;
        top_bar.is_expanded |= matches!(rewind_controller, RewindController::Mouse { .. });

        // TODO: Use one Instant per frame
        let now = Instant::now();

        top_bar.animation.transition(top_bar.is_expanded, now);
        top_bar
            .settings_animation
            .transition(top_bar.settings_active, now);

        if !top_bar.is_expanded {
            let progress_x = top_bar.bbox.size.width * player.percentage();
            draw_rect(
                quad_pipeline,
                &Rect::new((0.0, 0.0).into(), (progress_x, 5.0).into()),
                &BLUE,
            );
        }

        top_bar.bbox.origin.y = top_bar.animation.animate_bool(-h, 0.0, now);

        if top_bar.bbox.origin.y == -top_bar.bbox.size.height {
            return;
        }

        draw_rect(quad_pipeline, &top_bar.bbox, &BAR_BG);

        for f in [
            ProgressBar::update,
            Header::update,
            Looper::update,
            update_settings_card,
        ] {
            f(scene, ctx, &now);
        }

        if false {
            draw_debug_ui(&mut scene.top_bar.elements, &mut scene.quad_pipeline);
        }
    }
}

fn update_settings_card(scene: &mut PlayingScene, _ctx: &mut Context, now: &Instant) {
    let PlayingScene {
        top_bar,
        quad_pipeline,
        ..
    } = scene;

    let y = top_bar.bbox.origin.y;
    let h = top_bar.bbox.size.height;
    let w = top_bar.bbox.size.width;

    if top_bar.settings_animation.in_progress(*now) || top_bar.settings_animation.value {
        let card_w = 300.0;
        let card_x = top_bar.settings_animation.animate_bool(card_w, 0.0, *now);

        quad_pipeline.push(
            LAYER_FG,
            QuadInstance {
                position: [card_x + w - card_w, y + h + 1.0],
                size: [card_w, 100.0],
                color: BAR_BG.into_linear_rgba(),
                border_radius: [10.0, 0.0, 10.0, 0.0],
            },
        );
    }
}

fn draw_rect(quad_pipeline: &mut QuadPipeline, bbox: &Rect, color: &Color) {
    quad_pipeline.push(
        LAYER_FG,
        QuadInstance {
            position: bbox.origin.into(),
            size: bbox.size.into(),
            color: color.into_linear_rgba(),
            ..Default::default()
        },
    );
}

fn draw_debug_ui(elements: &mut nuon::ElementsMap<Msg>, quad_pipeline: &mut QuadPipeline) {
    for (element, dbg) in nuon::debug_ui::iter_elements(elements) {
        for (pos, size) in dbg.border {
            quad_pipeline.push(
                LAYER_FG,
                QuadInstance {
                    position: pos.into(),
                    size: size.into(),
                    color: [1.0, 1.0, 1.0, 0.5],
                    border_radius: [0.0; 4],
                },
            );
        }

        if element.hovered() {
            quad_pipeline.push(
                LAYER_FG,
                QuadInstance {
                    position: dbg.bounding_box.0.into(),
                    size: dbg.bounding_box.1.into(),
                    color: [1.0, 1.0, 1.0, 0.1],
                    border_radius: [0.0; 4],
                },
            );
        }
    }
}
