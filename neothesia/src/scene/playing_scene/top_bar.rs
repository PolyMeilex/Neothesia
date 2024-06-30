use std::time::{Duration, Instant};

use neothesia_core::{
    render::{QuadInstance, QuadPipeline, TextRenderer},
    utils::{Point, Rect, Size},
};
use wgpu_jumpstart::Color;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
};

use crate::{context::Context, utils::window::WindowState, NeothesiaEvent};

use super::{
    animation::{Animated, Animation, Easing},
    rewind_controller::RewindController,
    PlayingScene, EVENT_CAPTURED, EVENT_IGNORED, LAYER_FG,
};

mod button;
use button::Button;

#[derive(Debug, Clone)]
enum Msg {
    GoBack,
    PlayResume,
    LooperToggle,
    SettingsToggle,
    ProggresBarPressed,
    LoopTickDrag(LooperDrag),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LooperDrag {
    Start,
    End,
}

pub struct TopBar {
    elements: nuon::ElementsMap<Msg>,

    bbox: Rect<f32>,
    loop_tick_height: f32,

    animation: Animated<bool, Instant>,
    is_expanded: bool,

    settings_animation: Animation,
    drag: Option<LooperDrag>,

    pub loop_start: Duration,
    pub loop_end: Duration,

    bar_layout: nuon::TriRowLayout,

    back_button: Button,
    play_button: Button,
    loop_button: Button,
    settings_button: Button,

    progress_bar: nuon::ElementId,

    loop_start_tick: Rect,
    loop_start_tick_id: nuon::ElementId,

    loop_end_tick: Rect,
    loop_end_tick_id: nuon::ElementId,

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

fn new_button<M>(elements: &mut nuon::ElementsMap<M>, name: &'static str, on_click: M) -> Button {
    Button::new(elements.insert(nuon::ElementBuilder::new().name(name).on_click(on_click)))
}

impl TopBar {
    pub fn new() -> Self {
        let mut elements = nuon::ElementsMap::new();

        let mut back_button = new_button(&mut elements, "BackButton", Msg::GoBack);
        let mut play_button = new_button(&mut elements, "PlayButton", Msg::PlayResume);
        let mut loop_button = new_button(&mut elements, "LoopButton", Msg::LooperToggle);
        let mut settings_button = new_button(&mut elements, "SettingsButton", Msg::SettingsToggle);

        back_button.set_icon(left_arrow_icon());
        play_button.set_icon(pause_icon());
        loop_button.set_icon(repeat_icon());
        settings_button.set_icon(gear_icon());

        let progress_bar = elements.insert(
            nuon::ElementBuilder::new()
                .name("ProgressBar")
                .on_click(Msg::ProggresBarPressed),
        );

        let loop_start_tick_id = elements.insert(
            nuon::ElementBuilder::new()
                .name("LoopStartTick")
                .on_click(Msg::LoopTickDrag(LooperDrag::Start)),
        );

        let loop_end_tick_id = elements.insert(
            nuon::ElementBuilder::new()
                .name("LoopEndTick")
                .on_click(Msg::LoopTickDrag(LooperDrag::End)),
        );

        let bbox = Rect::new((0.0, 0.0).into(), (0.0, 45.0 + 30.0).into());

        Self {
            elements,

            bbox,
            loop_tick_height: 45.0 + 10.0,
            bar_layout: nuon::TriRowLayout::new(),

            back_button,
            play_button,
            loop_button,
            settings_button,

            drag: None,
            loop_start: Duration::ZERO,
            loop_end: Duration::ZERO,
            loop_active: false,
            animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),
            is_expanded: false,
            settings_animation: Animation::new(),
            progress_bar,
            loop_start_tick: Rect::default(),
            loop_end_tick_id,
            loop_end_tick: Rect::default(),
            loop_start_tick_id,
            settings_active: false,
        }
    }

    fn on_msg(scene: &mut PlayingScene, ctx: &mut Context, msg: Msg) {
        match msg {
            Msg::GoBack => {
                ctx.proxy.send_event(NeothesiaEvent::MainMenu).ok();
            }
            Msg::PlayResume => {
                scene.player.pause_resume();
            }
            Msg::LooperToggle => {
                scene.top_bar.loop_active = !scene.top_bar.loop_active;
                if scene.top_bar.loop_active {
                    scene.top_bar.loop_start = scene.player.time();
                    scene.top_bar.loop_end = scene.top_bar.loop_start + Duration::from_secs(3);
                };
            }
            Msg::SettingsToggle => {
                scene.top_bar.settings_active = !scene.top_bar.settings_active;
            }
            Msg::LoopTickDrag(side) => {
                scene.top_bar.drag = Some(side);
            }
            Msg::ProggresBarPressed => {
                if !scene.rewind_controller.is_rewinding() {
                    scene
                        .rewind_controller
                        .start_mouse_rewind(&mut scene.player);

                    let x = ctx.window_state.cursor_logical_position.x;
                    let w = ctx.window_state.logical_size.width;

                    let p = x / w;
                    scene.player.set_percentage_time(p);
                    scene.keyboard.reset_notes();
                }
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
                scene.top_bar.bar_layout.invalidate();
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
                    .hovered_element()
                    .and_then(|(_, e)| e.on_click())
                {
                    Self::on_msg(scene, ctx, msg.clone());
                    return EVENT_CAPTURED;
                }
            }
            (ElementState::Released, MouseButton::Left) => {
                scene.top_bar.drag = None;

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

        scene.top_bar.elements.update_cursor_pos((x, y).into());

        match scene.top_bar.drag {
            Some(LooperDrag::Start) => {
                scene.top_bar.loop_start = scene.player.percentage_to_time(x / w);
            }
            Some(LooperDrag::End) => {
                scene.top_bar.loop_end = scene.player.percentage_to_time(x / w);
            }
            None => {}
        }
    }

    #[profiling::function]
    pub fn update(scene: &mut PlayingScene, window_state: &WindowState, text: &mut TextRenderer) {
        let PlayingScene {
            top_bar,
            quad_pipeline,
            ref player,
            ref rewind_controller,
            ..
        } = scene;

        top_bar.bbox.size.width = window_state.logical_size.width;

        let h = top_bar.bbox.size.height;
        let is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        top_bar.is_expanded = is_hovered;
        top_bar.is_expanded |= top_bar.settings_active;
        top_bar.is_expanded |= matches!(rewind_controller, RewindController::Mouse { .. });

        // TODO: Use one Instant per frame
        let now = Instant::now();

        if top_bar.animation.value != top_bar.is_expanded {
            top_bar.animation.transition(top_bar.is_expanded, now);
        }
        top_bar.settings_animation.update(top_bar.settings_active);

        if !top_bar.is_expanded {
            let progress_x = top_bar.bbox.size.width * player.percentage();
            draw_rect(
                quad_pipeline,
                &Rect::new((0.0, 0.0).into(), (progress_x, 5.0).into()),
                &BLUE,
            );
        }

        top_bar.bbox.origin.y = top_bar.animation.animate(-h, 0.0, now);

        if top_bar.bbox.origin.y == -top_bar.bbox.size.height {
            return;
        }

        draw_rect(quad_pipeline, &top_bar.bbox, &BAR_BG);

        for f in [
            update_proggress_bar,
            update_buttons,
            update_looper,
            update_settings_card,
        ] {
            f(scene, text, &now);
        }
    }
}

fn update_proggress_bar(scene: &mut PlayingScene, _text: &mut TextRenderer, _now: &Instant) {
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
    top_bar.elements.update(top_bar.progress_bar, rect);

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
            &Rect::new((x, y).into(), (1.0, h).into()),
            &color,
        );
    }
}

fn update_button_icons(scene: &mut PlayingScene) {
    let PlayingScene {
        top_bar, player, ..
    } = scene;

    top_bar.play_button.set_icon(if player.is_paused() {
        play_icon()
    } else {
        pause_icon()
    });

    top_bar
        .settings_button
        .set_icon(if top_bar.settings_active {
            gear_fill_icon()
        } else {
            gear_icon()
        });
}

fn update_buttons(scene: &mut PlayingScene, text: &mut TextRenderer, _now: &Instant) {
    update_button_icons(scene);

    let PlayingScene {
        top_bar,
        quad_pipeline,
        ..
    } = scene;

    let y = top_bar.bbox.origin.y;
    let w = top_bar.bbox.size.width;

    let (_back_id,) = top_bar.bar_layout.start.once(|row| {
        (
            row.push(30.0),
            //
        )
    });

    top_bar.bar_layout.center.once(|_row| {});

    let (_play, _loop, _settings) = top_bar.bar_layout.end.once(|row| {
        (
            row.push(30.0),
            row.push(30.0),
            row.push(30.0),
            //
        )
    });

    top_bar.bar_layout.resolve(0.0, w);

    let start_row = top_bar.bar_layout.start.items();
    let _center_row = top_bar.bar_layout.center.items();
    let end_row = top_bar.bar_layout.end.items();

    for (btn, item) in [&mut top_bar.back_button].into_iter().zip(start_row) {
        btn.update(
            &mut top_bar.elements,
            Rect::new((item.x, y).into(), (item.width, 30.0).into()),
        );
    }

    for (btn, item) in [
        &mut top_bar.play_button,
        &mut top_bar.loop_button,
        &mut top_bar.settings_button,
    ]
    .into_iter()
    .zip(end_row)
    {
        btn.update(
            &mut top_bar.elements,
            Rect::new((item.x, y).into(), (item.width, 30.0).into()),
        );
    }

    for btn in [
        &top_bar.back_button,
        &top_bar.play_button,
        &top_bar.loop_button,
        &top_bar.settings_button,
    ] {
        btn.draw(quad_pipeline, text);
    }
}

fn update_looper(scene: &mut PlayingScene, _text: &mut TextRenderer, now: &Instant) {
    let PlayingScene {
        top_bar,
        quad_pipeline,
        ref player,
        ..
    } = scene;

    if !top_bar.loop_active {
        top_bar
            .elements
            .update(top_bar.loop_start_tick_id, nuon::Rect::zero());
        top_bar
            .elements
            .update(top_bar.loop_end_tick_id, nuon::Rect::zero());
        return;
    }

    let y = top_bar
        .animation
        .animate(-top_bar.bbox.size.height - 30.0, 30.0, *now);
    let alpha = top_bar.animation.animate(0.0, 1.0, *now);

    let h = top_bar.loop_tick_height;
    let w = top_bar.bbox.size.width;

    let tick_size = Size::new(5.0, h);
    let tick_pos = |start: &Duration| Point::new(player.time_to_percentage(start) * w, y);

    top_bar.loop_start_tick = Rect::new(tick_pos(&top_bar.loop_start), tick_size);
    top_bar.loop_end_tick = Rect::new(tick_pos(&top_bar.loop_end), tick_size);

    let start_hovered = top_bar
        .elements
        .get(top_bar.loop_start_tick_id)
        .map(|e| e.hovered())
        .unwrap_or(false)
        || top_bar.drag == Some(LooperDrag::Start);
    let end_hovered = top_bar
        .elements
        .get(top_bar.loop_end_tick_id)
        .map(|e| e.hovered())
        .unwrap_or(false)
        || top_bar.drag == Some(LooperDrag::End);

    let mut start_color = if start_hovered { WHITE } else { LOOPER };
    let mut end_color = if end_hovered { WHITE } else { LOOPER };

    start_color.a *= alpha;
    end_color.a *= alpha;

    let color = Color {
        a: 0.35 * alpha,
        ..LOOPER
    };

    let length = top_bar.loop_end_tick.origin.x - top_bar.loop_start_tick.origin.x;

    let mut bg_box = top_bar.loop_start_tick;
    bg_box.size.width = length;

    draw_rect(quad_pipeline, &bg_box, &color);
    draw_rect(quad_pipeline, &top_bar.loop_start_tick, &start_color);
    draw_rect(quad_pipeline, &top_bar.loop_end_tick, &end_color);

    top_bar
        .elements
        .update(top_bar.loop_start_tick_id, top_bar.loop_start_tick);
    top_bar
        .elements
        .update(top_bar.loop_end_tick_id, top_bar.loop_end_tick);
}

fn update_settings_card(scene: &mut PlayingScene, _text: &mut TextRenderer, _now: &Instant) {
    let PlayingScene {
        top_bar,
        quad_pipeline,
        ..
    } = scene;

    let settings_animation = top_bar.settings_animation.expo_out(top_bar.settings_active);

    let y = top_bar.bbox.origin.y;
    let h = top_bar.bbox.size.height;
    let w = top_bar.bbox.size.width;

    if !top_bar.settings_animation.is_done() {
        let card_w = 300.0;
        let card_x = card_w - (settings_animation * card_w);

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
