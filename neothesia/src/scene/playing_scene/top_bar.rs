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
    animation, rewind_controller::RewindController, PlayingScene, EVENT_CAPTURED, EVENT_IGNORED,
};

#[derive(Default, Clone, Copy)]
enum Element {
    StartTick,
    EndTick,
    RepeatButton,
    BackButton,
    PlayButton,
    #[default]
    None,
}

pub struct TopBar {
    height: f32,
    loop_tick_height: f32,

    animation: f32,
    drag: Element,
    hovered: Element,

    pub loop_start: Duration,
    pub loop_end: Duration,

    back_button: Button,
    play_button: Button,
    loop_button: Button,
    loop_start_tick: Bbox,
    loop_end_tick: Bbox,

    pub loop_active: bool,
}

#[derive(Default)]
struct Button {
    bbox: Bbox,
    element: Element,
    is_hovered: bool,
    icon: &'static str,
}

impl Button {
    fn new(element: Element) -> Self {
        Self {
            element,
            bbox: Bbox::new(Point::new(0.0, 0.0), Size::new(30.0, 30.0)),
            is_hovered: false,
            icon: "",
        }
    }

    fn set_x(&mut self, x: f32) -> &mut Self {
        self.bbox.pos.x = x;
        self
    }

    fn set_y(&mut self, y: f32) -> &mut Self {
        self.bbox.pos.y = y;
        self
    }

    fn set_hovered(&mut self, hovered: bool) -> &mut Self {
        self.is_hovered = hovered;
        self
    }

    fn set_icon(&mut self, icon: &'static str) -> &mut Self {
        self.icon = icon;
        self
    }

    fn bbox_with_type(&self) -> (&Bbox, Element) {
        (&self.bbox, self.element)
    }

    fn draw(&mut self, quad_pipeline: &mut QuadPipeline, text: &mut TextRenderer) {
        let color = if self.is_hovered {
            BUTTON_HOVER
        } else {
            BAR_BG
        }
        .into_linear_rgba();

        quad_pipeline.push(QuadInstance {
            position: self.bbox.pos.into(),
            size: self.bbox.size.into(),
            color,
            border_radius: [5.0; 4],
        });

        let icon_size = 20.0;
        text.queue_icon(
            self.bbox.x() + (self.bbox.w() - icon_size) / 2.0,
            self.bbox.y() + (self.bbox.h() - icon_size) / 2.0,
            icon_size,
            self.icon,
        );
    }
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
            height: 45.0 + 30.0,
            loop_tick_height: 45.0 + 10.0,
            loop_button: Button::new(Element::RepeatButton),
            back_button: Button::new(Element::BackButton),
            play_button: Button::new(Element::PlayButton),
            drag: Element::None,
            hovered: Element::None,
            loop_start: Duration::ZERO,
            loop_end: Duration::ZERO,
            loop_active: false,
            animation: 0.0,
            loop_start_tick: Bbox::default(),
            loop_end_tick: Bbox::default(),
        }
    }

    fn is_fully_collapsed(&self) -> bool {
        self.animation == 0.0
    }

    fn hovered(&self, x: f32, y: f32) -> Element {
        [
            self.loop_button.bbox_with_type(),
            self.back_button.bbox_with_type(),
            self.play_button.bbox_with_type(),
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
                        && pos.y < scene.top_bar.height
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

        let h = top_bar.height;
        let w = window_state.logical_size.width;
        let progress_x = w * player.percentage();

        let mut is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        if let RewindController::Mouse { .. } = rewind_controller {
            is_hovered = true;
        }

        if is_hovered {
            top_bar.animation += 0.04;
        } else {
            top_bar.animation -= 0.1;
        }

        top_bar.animation = top_bar.animation.min(1.0);
        top_bar.animation = top_bar.animation.max(0.0);

        if !is_hovered {
            fg_quad_pipeline.push(QuadInstance {
                position: [0.0, 0.0],
                size: [progress_x, 5.0],
                color: BLUE.into_linear_rgba(),
                ..Default::default()
            });
        }

        if top_bar.is_fully_collapsed() {
            return;
        }

        let bar_animation = if is_hovered {
            animation::expo_out(top_bar.animation)
        } else {
            top_bar.animation
        };

        let y = -h + (bar_animation * h);

        fg_quad_pipeline.push(QuadInstance {
            position: [0.0, y],
            size: [w, h],
            color: BAR_BG.into_linear_rgba(),
            ..Default::default()
        });

        let progress_x = w * player.percentage();
        fg_quad_pipeline.push(QuadInstance {
            position: [0.0, y + 30.0],
            size: [progress_x, h - 30.0],
            color: BLUE.into_linear_rgba(),
            ..Default::default()
        });

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

            fg_quad_pipeline.push(QuadInstance {
                position: [x, y + 30.0],
                size: [1.0, h - 30.0],
                color: color.into_linear_rgba(),
                ..Default::default()
            });
        }

        update_buttons(scene, y, w, text);
        update_looper(scene, w, bar_animation);
    }
}

fn update_buttons(scene: &mut PlayingScene, y: f32, w: f32, text: &mut TextRenderer) {
    let PlayingScene {
        top_bar,
        fg_quad_pipeline,
        ..
    } = scene;

    top_bar
        .back_button
        .set_x(0.0)
        .set_y(y)
        .set_hovered(matches!(top_bar.hovered, Element::BackButton))
        .set_icon(left_arrow_icon())
        .draw(fg_quad_pipeline, text);

    top_bar
        .loop_button
        .set_x(w - 30.0)
        .set_y(y)
        .set_hovered(matches!(top_bar.hovered, Element::RepeatButton))
        .set_icon(repeat_icon())
        .draw(fg_quad_pipeline, text);

    top_bar
        .play_button
        .set_x(w - 30.0 * 2.0)
        .set_y(y)
        .set_hovered(matches!(top_bar.hovered, Element::PlayButton))
        .set_icon(if scene.player.is_paused() {
            play_icon()
        } else {
            pause_icon()
        })
        .draw(fg_quad_pipeline, text);
}

fn update_looper(scene: &mut PlayingScene, w: f32, animation: f32) {
    let PlayingScene {
        top_bar,
        fg_quad_pipeline,
        ref player,
        ..
    } = scene;

    if !top_bar.loop_active {
        return;
    }

    let h = top_bar.loop_tick_height;
    let offset = 30.0 + h;
    let y = 30.0 - offset + (animation * offset);

    top_bar.loop_start_tick = Bbox::new(
        Point {
            x: player.time_to_percentage(&top_bar.loop_start) * w,
            y,
        },
        Size::new(5.0, h),
    );
    top_bar.loop_end_tick = Bbox::new(
        Point {
            x: player.time_to_percentage(&top_bar.loop_end) * w,
            y,
        },
        Size::new(5.0, h),
    );

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

fn draw_rect(quad_pipeline: &mut QuadPipeline, bbox: &Bbox, color: &Color) {
    quad_pipeline.push(QuadInstance {
        position: bbox.pos.into(),
        size: bbox.size.into(),
        color: color.into_linear_rgba(),
        ..Default::default()
    });
}
