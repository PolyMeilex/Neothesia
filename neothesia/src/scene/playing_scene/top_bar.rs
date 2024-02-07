use std::time::Duration;

use neothesia_core::render::{QuadInstance, TextRenderer};
use wgpu_jumpstart::Color;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
};

use crate::{target::Target, utils::window::WindowState, NeothesiaEvent};

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

#[derive(Default, Clone, Copy)]
struct ElementInfo {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl ElementInfo {
    fn contains(&self, x: f32, y: f32) -> bool {
        (self.x..(self.x + self.w)).contains(&x) && (self.y..(self.y + self.h)).contains(&y)
    }

    fn pos(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    fn size(&self) -> [f32; 2] {
        [self.w, self.h]
    }
}

#[derive(Default)]
pub struct TopBar {
    height: f32,
    loop_tick_height: f32,

    animation: f32,
    drag: Element,
    hovered: Element,

    pub loop_start: Duration,
    pub loop_end: Duration,

    back_button: ElementInfo,
    play_button: ElementInfo,
    loop_button: ElementInfo,
    loop_start_tick: ElementInfo,
    loop_end_tick: ElementInfo,

    pub loop_active: bool,
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
            ..Default::default()
        }
    }

    fn is_fully_colapsed(&self) -> bool {
        self.animation == 0.0
    }

    fn hovered(&self, x: f32, y: f32) -> Element {
        [
            (&self.loop_button, Element::RepeatButton),
            (&self.back_button, Element::BackButton),
            (&self.play_button, Element::PlayButton),
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
        target: &mut Target,
        event: &WindowEvent,
    ) -> bool {
        match &event {
            WindowEvent::MouseInput { state, button, .. } => {
                return Self::handle_mouse_input(scene, target, state, button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                Self::handle_cursor_moved(scene, target, position);
            }
            _ => {}
        }

        EVENT_IGNORED
    }

    fn handle_mouse_input(
        scene: &mut PlayingScene,
        target: &mut Target,
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
                    target.proxy.send_event(NeothesiaEvent::MainMenu).ok();
                    return EVENT_CAPTURED;
                }
                Element::StartTick | Element::EndTick => {
                    scene.top_bar.drag = scene.top_bar.hovered;
                    return EVENT_CAPTURED;
                }
                _ => {
                    let pos = &target.window_state.cursor_logical_position;

                    if pos.y > 30.0
                        && pos.y < scene.top_bar.height
                        && !scene.rewind_controler.is_rewinding()
                    {
                        scene.rewind_controler.start_mouse_rewind(&mut scene.player);

                        let x = target.window_state.cursor_logical_position.x;
                        let w = target.window_state.logical_size.width;

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

                if let RewindController::Mouse { .. } = scene.rewind_controler {
                    scene.rewind_controler.stop_rewind(&mut scene.player);
                }
            }
            _ => {}
        }

        EVENT_IGNORED
    }

    fn handle_cursor_moved(
        scene: &mut PlayingScene,
        target: &mut Target,
        position: &PhysicalPosition<f64>,
    ) {
        let x = position
            .to_logical::<f32>(target.window_state.scale_factor)
            .x;
        let y = position
            .to_logical::<f32>(target.window_state.scale_factor)
            .y;
        let w = target.window_state.logical_size.width;

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
        let top_bar = &mut scene.top_bar;
        let quad_pipeline = &mut scene.fg_quad_pipeline;
        let player = &scene.player;
        let rewind_controler = &scene.rewind_controler;

        let h = top_bar.height;
        let w = window_state.logical_size.width;
        let progress_x = w * player.percentage();

        let mut is_hovered = if top_bar.is_fully_colapsed() {
            window_state.cursor_logical_position.y < h
        } else {
            window_state.cursor_logical_position.y < h * 3.0
        };
        is_hovered |= rewind_controler.is_rewinding();

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

        if top_bar.is_fully_colapsed() {
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
            position: [0.0, y + 30.0],
            size: [progress_x, h - 30.0],
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
                position: [x, y + 30.0],
                size: [1.0, h - 30.0],
                color: color.into_linear_rgba(),
                ..Default::default()
            });
        }

        update_loop_button(scene, y, w, text);
        update_looper(scene, w, bar_animation);
    }
}

fn update_loop_button(scene: &mut PlayingScene, y: f32, w: f32, text: &mut TextRenderer) {
    let top_bar = &mut scene.top_bar;

    top_bar.loop_button = ElementInfo {
        x: w - 30.0,
        y,
        w: 30.0,
        h: 30.0,
    };

    let color = if let Element::RepeatButton = top_bar.hovered {
        BUTTON_HOVER
    } else {
        BAR_BG
    };

    scene.fg_quad_pipeline.instances().push(QuadInstance {
        position: top_bar.loop_button.pos(),
        size: top_bar.loop_button.size(),
        color: color.into_linear_rgba(),
        border_radius: [5.0; 4],
    });

    let icon_size = 20.0;
    text.queue_icon(
        top_bar.loop_button.x + (top_bar.loop_button.w - icon_size) / 2.0,
        top_bar.loop_button.y + (top_bar.loop_button.h - icon_size) / 2.0,
        icon_size,
        repeat_icon(),
    );

    {
        top_bar.play_button = ElementInfo {
            x: top_bar.loop_button.x - 30.0,
            ..top_bar.loop_button
        };

        let color = if let Element::PlayButton = top_bar.hovered {
            BUTTON_HOVER
        } else {
            BAR_BG
        };
        scene.fg_quad_pipeline.instances().push(QuadInstance {
            position: top_bar.play_button.pos(),
            size: top_bar.play_button.size(),
            color: color.into_linear_rgba(),
            border_radius: [5.0; 4],
        });
        text.queue_icon(
            top_bar.play_button.x + (top_bar.play_button.w - icon_size) / 2.0,
            top_bar.play_button.y + (top_bar.play_button.h - icon_size) / 2.0,
            icon_size,
            if scene.player.is_paused() {
                play_icon()
            } else {
                pause_icon()
            },
        );
    }

    {
        top_bar.back_button = ElementInfo {
            x: 0.0,
            ..top_bar.loop_button
        };

        let color = if let Element::BackButton = top_bar.hovered {
            BUTTON_HOVER
        } else {
            BAR_BG
        };
        scene.fg_quad_pipeline.instances().push(QuadInstance {
            position: top_bar.back_button.pos(),
            size: top_bar.back_button.size(),
            color: color.into_linear_rgba(),
            border_radius: [5.0; 4],
        });

        text.queue_icon(
            top_bar.back_button.x + (top_bar.back_button.w - icon_size) / 2.0,
            top_bar.back_button.y + (top_bar.back_button.h - icon_size) / 2.0,
            icon_size,
            left_arrow_icon(),
        );
    }
}

fn update_looper(scene: &mut PlayingScene, w: f32, animation: f32) {
    let top_bar = &mut scene.top_bar;
    let quad_pipeline = &mut scene.fg_quad_pipeline;

    let h = top_bar.loop_tick_height;

    top_bar.loop_start_tick = ElementInfo {
        x: scene.player.time_to_percentage(&top_bar.loop_start) * w,
        y: 30.0,
        w: 5.0,
        h,
    };
    top_bar.loop_end_tick = ElementInfo {
        x: scene.player.time_to_percentage(&top_bar.loop_end) * w,
        y: 30.0,
        w: 5.0,
        h,
    };

    if !top_bar.loop_active {
        return;
    }

    let offset = top_bar.loop_start_tick.y + h;
    top_bar.loop_start_tick.y += -offset + (animation * offset);
    top_bar.loop_end_tick.y = top_bar.loop_start_tick.y;

    let (start_color, end_color) = match (top_bar.hovered, top_bar.drag) {
        (Element::StartTick, _) | (_, Element::StartTick) => (WHITE, LOOPER),
        (Element::EndTick, _) | (_, Element::EndTick) => (LOOPER, WHITE),
        _ => (LOOPER, LOOPER),
    };

    let color = Color { a: 0.35, ..LOOPER };

    let length = top_bar.loop_end_tick.x - top_bar.loop_start_tick.x;

    quad_pipeline.instances().push(QuadInstance {
        position: top_bar.loop_start_tick.pos(),
        size: [length, top_bar.loop_start_tick.h],
        color: color.into_linear_rgba(),
        ..Default::default()
    });

    quad_pipeline.instances().push(QuadInstance {
        position: top_bar.loop_start_tick.pos(),
        size: top_bar.loop_start_tick.size(),
        color: start_color.into_linear_rgba(),
        ..Default::default()
    });
    quad_pipeline.instances().push(QuadInstance {
        position: top_bar.loop_end_tick.pos(),
        size: top_bar.loop_end_tick.size(),
        color: end_color.into_linear_rgba(),
        ..Default::default()
    });
}
