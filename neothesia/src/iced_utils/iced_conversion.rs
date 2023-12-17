#![allow(dead_code)]

//! https://github.com/hecrj/iced/blob/master/winit/src/conversion.rs
use iced_core::{
    keyboard::{self},
    mouse, touch, window, Event, Point,
};

/// The position of a window in a given screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// The platform-specific default position for a new window.
    Default,
    /// The window is completely centered on the screen.
    Centered,
    /// The window is positioned with specific coordinates: `(X, Y)`.
    ///
    /// When the decorations of the window are enabled, Windows 10 will add some
    /// invisible padding to the window. This padding gets included in the
    /// position. So if you have decorations enabled and want the window to be
    /// at (0, 0) you would have to set the position to
    /// `(PADDING_X, PADDING_Y)`.
    Specific(i32, i32),
}

impl Default for Position {
    fn default() -> Self {
        Self::Default
    }
}

/// The mode of a window-based application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The application appears in its own window.
    Windowed,

    /// The application takes the whole screen of its current monitor.
    Fullscreen,

    /// The application is hidden
    Hidden,
}

/// Converts a winit window event into an iced event.
pub fn window_event(
    event: &winit::event::WindowEvent,
    scale_factor: f64,
    modifiers: winit::keyboard::ModifiersState,
) -> Option<Event> {
    use winit::event::WindowEvent;

    let id = iced_core::window::Id::MAIN;

    match event {
        WindowEvent::Resized(new_size) => {
            let logical_size = new_size.to_logical(scale_factor);

            Some(Event::Window(
                id,
                window::Event::Resized {
                    width: logical_size.width,
                    height: logical_size.height,
                },
            ))
        }
        WindowEvent::CloseRequested => Some(Event::Window(id, window::Event::CloseRequested)),
        WindowEvent::CursorMoved { position, .. } => {
            let position = position.to_logical::<f64>(scale_factor);

            Some(Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(position.x as f32, position.y as f32),
            }))
        }
        WindowEvent::CursorEntered { .. } => Some(Event::Mouse(mouse::Event::CursorEntered)),
        WindowEvent::CursorLeft { .. } => Some(Event::Mouse(mouse::Event::CursorLeft)),
        WindowEvent::MouseInput { button, state, .. } => {
            let button = mouse_button(*button);

            Some(Event::Mouse(match state {
                winit::event::ElementState::Pressed => mouse::Event::ButtonPressed(button),
                winit::event::ElementState::Released => mouse::Event::ButtonReleased(button),
            }))
        }
        WindowEvent::MouseWheel { delta, .. } => match delta {
            winit::event::MouseScrollDelta::LineDelta(delta_x, delta_y) => {
                Some(Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines {
                        x: *delta_x,
                        y: *delta_y,
                    },
                }))
            }
            winit::event::MouseScrollDelta::PixelDelta(position) => {
                Some(Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels {
                        x: position.x as f32,
                        y: position.y as f32,
                    },
                }))
            }
        },
        WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key,
                    state,
                    // text,
                    ..
                },
            ..
        } => Some(Event::Keyboard({
            let key_code = key_code(logical_key);
            let modifiers = self::modifiers(modifiers);

            match state {
                winit::event::ElementState::Pressed => keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                },
                winit::event::ElementState::Released => keyboard::Event::KeyReleased {
                    key_code,
                    modifiers,
                },
            }
        })),
        WindowEvent::ModifiersChanged(new_modifiers) => Some(Event::Keyboard(
            keyboard::Event::ModifiersChanged(self::modifiers(new_modifiers.state())),
        )),
        WindowEvent::Focused(focused) => Some(Event::Window(
            id,
            if *focused {
                window::Event::Focused
            } else {
                window::Event::Unfocused
            },
        )),
        WindowEvent::HoveredFile(path) => {
            Some(Event::Window(id, window::Event::FileHovered(path.clone())))
        }
        WindowEvent::DroppedFile(path) => {
            Some(Event::Window(id, window::Event::FileDropped(path.clone())))
        }
        WindowEvent::HoveredFileCancelled => {
            Some(Event::Window(id, window::Event::FilesHoveredLeft))
        }
        WindowEvent::Touch(touch) => Some(Event::Touch(touch_event(*touch, scale_factor))),
        WindowEvent::Moved(position) => {
            let winit::dpi::LogicalPosition { x, y } = position.to_logical(scale_factor);

            Some(Event::Window(id, window::Event::Moved { x, y }))
        }
        _ => None,
    }
}

/// Converts a [`Position`] to a [`winit`] logical position for a given monitor.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn position(
    monitor: Option<&winit::monitor::MonitorHandle>,
    (width, height): (u32, u32),
    position: Position,
) -> Option<winit::dpi::Position> {
    match position {
        Position::Default => None,
        Position::Specific(x, y) => {
            Some(winit::dpi::Position::Logical(winit::dpi::LogicalPosition {
                x: f64::from(x),
                y: f64::from(y),
            }))
        }
        Position::Centered => {
            if let Some(monitor) = monitor {
                let start = monitor.position();

                let resolution: winit::dpi::LogicalSize<f64> =
                    monitor.size().to_logical(monitor.scale_factor());

                let centered: winit::dpi::PhysicalPosition<i32> = winit::dpi::LogicalPosition {
                    x: (resolution.width - f64::from(width)) / 2.0,
                    y: (resolution.height - f64::from(height)) / 2.0,
                }
                .to_physical(monitor.scale_factor());

                Some(winit::dpi::Position::Physical(
                    winit::dpi::PhysicalPosition {
                        x: start.x + centered.x,
                        y: start.y + centered.y,
                    },
                ))
            } else {
                None
            }
        }
    }
}

/// Converts a [`Mode`] to a [`winit`] fullscreen mode.
///
/// [`winit`]: https://github.com/rust-windowing/winit
pub fn fullscreen(
    monitor: Option<winit::monitor::MonitorHandle>,
    mode: Mode,
) -> Option<winit::window::Fullscreen> {
    match mode {
        Mode::Windowed | Mode::Hidden => None,
        Mode::Fullscreen => Some(winit::window::Fullscreen::Borderless(monitor)),
    }
}

/// Converts a [`Mode`] to a visibility flag.
pub fn visible(mode: Mode) -> bool {
    match mode {
        Mode::Windowed | Mode::Fullscreen => true,
        Mode::Hidden => false,
    }
}

/// Converts a `MouseCursor` from [`iced_native`] to a [`winit`] cursor icon.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn mouse_interaction(interaction: mouse::Interaction) -> winit::window::CursorIcon {
    use mouse::Interaction;

    match interaction {
        Interaction::Idle => winit::window::CursorIcon::Default,
        Interaction::Pointer => winit::window::CursorIcon::Pointer,
        Interaction::Working => winit::window::CursorIcon::Progress,
        Interaction::Grab => winit::window::CursorIcon::Grab,
        Interaction::Grabbing => winit::window::CursorIcon::Grabbing,
        Interaction::Crosshair => winit::window::CursorIcon::Crosshair,
        Interaction::Text => winit::window::CursorIcon::Text,
        Interaction::ResizingHorizontally => winit::window::CursorIcon::EwResize,
        Interaction::ResizingVertically => winit::window::CursorIcon::NsResize,
        Interaction::NotAllowed => winit::window::CursorIcon::NotAllowed,
    }
}

/// Converts a `MouseButton` from [`winit`] to an [`iced_native`] mouse button.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn mouse_button(mouse_button: winit::event::MouseButton) -> mouse::Button {
    match mouse_button {
        winit::event::MouseButton::Left => mouse::Button::Left,
        winit::event::MouseButton::Right => mouse::Button::Right,
        winit::event::MouseButton::Middle => mouse::Button::Middle,
        winit::event::MouseButton::Back => mouse::Button::Other(99),
        winit::event::MouseButton::Forward => mouse::Button::Other(98),
        // winit::event::MouseButton::Back => mouse::Button::Back,
        // winit::event::MouseButton::Forward => mouse::Button::Forward,
        winit::event::MouseButton::Other(other) => mouse::Button::Other(other),
    }
}

/// Converts some `ModifiersState` from [`winit`] to an [`iced_native`]
/// modifiers state.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn modifiers(modifiers: winit::keyboard::ModifiersState) -> keyboard::Modifiers {
    let mut result = keyboard::Modifiers::empty();

    result.set(keyboard::Modifiers::SHIFT, modifiers.shift_key());
    result.set(keyboard::Modifiers::CTRL, modifiers.control_key());
    result.set(keyboard::Modifiers::ALT, modifiers.alt_key());
    result.set(keyboard::Modifiers::LOGO, modifiers.super_key());

    result
}

/// Converts a physical cursor position to a logical `Point`.
pub fn cursor_position(position: winit::dpi::PhysicalPosition<f64>, scale_factor: f64) -> Point {
    let logical_position = position.to_logical(scale_factor);

    Point::new(logical_position.x, logical_position.y)
}

/// Converts a `Touch` from [`winit`] to an [`iced_native`] touch event.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn touch_event(touch: winit::event::Touch, scale_factor: f64) -> touch::Event {
    let id = touch::Finger(touch.id);
    let position = {
        let location = touch.location.to_logical::<f64>(scale_factor);

        Point::new(location.x as f32, location.y as f32)
    };

    match touch.phase {
        winit::event::TouchPhase::Started => touch::Event::FingerPressed { id, position },
        winit::event::TouchPhase::Moved => touch::Event::FingerMoved { id, position },
        winit::event::TouchPhase::Ended => touch::Event::FingerLifted { id, position },
        winit::event::TouchPhase::Cancelled => touch::Event::FingerLost { id, position },
    }
}

/// Converts a `VirtualKeyCode` from [`winit`] to an [`iced_native`] key code.
///
/// [`winit`]: https://github.com/rust-windowing/winit
/// [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
pub fn key_code(key: &winit::keyboard::Key) -> keyboard::KeyCode {
    use keyboard::KeyCode;
    use winit::keyboard::NamedKey;

    match key {
        winit::keyboard::Key::Character(c) => match c.as_str() {
            "1" => KeyCode::Key1,
            "2" => KeyCode::Key2,
            "3" => KeyCode::Key3,
            "4" => KeyCode::Key4,
            "5" => KeyCode::Key5,
            "6" => KeyCode::Key6,
            "7" => KeyCode::Key7,
            "8" => KeyCode::Key8,
            "9" => KeyCode::Key9,
            "0" => KeyCode::Key0,
            "a" => KeyCode::A,
            "b" => KeyCode::B,
            "c" => KeyCode::C,
            "d" => KeyCode::D,
            "e" => KeyCode::E,
            "f" => KeyCode::F,
            "g" => KeyCode::G,
            "h" => KeyCode::H,
            "i" => KeyCode::I,
            "j" => KeyCode::J,
            "k" => KeyCode::K,
            "l" => KeyCode::L,
            "m" => KeyCode::M,
            "n" => KeyCode::N,
            "o" => KeyCode::O,
            "p" => KeyCode::P,
            "q" => KeyCode::Q,
            "r" => KeyCode::R,
            "s" => KeyCode::S,
            "t" => KeyCode::T,
            "u" => KeyCode::U,
            "v" => KeyCode::V,
            "w" => KeyCode::W,
            "x" => KeyCode::X,
            "y" => KeyCode::Y,
            "z" => KeyCode::Z,
            _ => KeyCode::Unlabeled,
        },
        winit::keyboard::Key::Named(named_key) => match named_key {
            NamedKey::Escape => KeyCode::Escape,
            NamedKey::F1 => KeyCode::F1,
            NamedKey::F2 => KeyCode::F2,
            NamedKey::F3 => KeyCode::F3,
            NamedKey::F4 => KeyCode::F4,
            NamedKey::F5 => KeyCode::F5,
            NamedKey::F6 => KeyCode::F6,
            NamedKey::F7 => KeyCode::F7,
            NamedKey::F8 => KeyCode::F8,
            NamedKey::F9 => KeyCode::F9,
            NamedKey::F10 => KeyCode::F10,
            NamedKey::F11 => KeyCode::F11,
            NamedKey::F12 => KeyCode::F12,
            NamedKey::F13 => KeyCode::F13,
            NamedKey::F14 => KeyCode::F14,
            NamedKey::F15 => KeyCode::F15,
            NamedKey::F16 => KeyCode::F16,
            NamedKey::F17 => KeyCode::F17,
            NamedKey::F18 => KeyCode::F18,
            NamedKey::F19 => KeyCode::F19,
            NamedKey::F20 => KeyCode::F20,
            NamedKey::F21 => KeyCode::F21,
            NamedKey::F22 => KeyCode::F22,
            NamedKey::F23 => KeyCode::F23,
            NamedKey::F24 => KeyCode::F24,
            NamedKey::PrintScreen => KeyCode::Snapshot,
            NamedKey::ScrollLock => KeyCode::Scroll,
            NamedKey::Pause => KeyCode::Pause,
            NamedKey::Insert => KeyCode::Insert,
            NamedKey::Home => KeyCode::Home,
            NamedKey::Delete => KeyCode::Delete,
            NamedKey::End => KeyCode::End,
            NamedKey::PageDown => KeyCode::PageDown,
            NamedKey::PageUp => KeyCode::PageUp,
            NamedKey::ArrowLeft => KeyCode::Left,
            NamedKey::ArrowUp => KeyCode::Up,
            NamedKey::ArrowRight => KeyCode::Right,
            NamedKey::ArrowDown => KeyCode::Down,
            NamedKey::Backspace => KeyCode::Backspace,
            NamedKey::Enter => KeyCode::Enter,
            NamedKey::Space => KeyCode::Space,
            NamedKey::Compose => KeyCode::Compose,
            NamedKey::NumLock => KeyCode::Numlock,
            NamedKey::AppSwitch => KeyCode::Apps,
            NamedKey::Convert => KeyCode::Convert,
            NamedKey::LaunchMail => KeyCode::Mail,
            NamedKey::MediaApps => KeyCode::MediaSelect,
            NamedKey::MediaStop => KeyCode::MediaStop,
            NamedKey::AudioVolumeMute => KeyCode::Mute,
            NamedKey::MediaStepForward => KeyCode::NavigateForward,
            NamedKey::MediaStepBackward => KeyCode::NavigateBackward,
            NamedKey::MediaSkipForward => KeyCode::NextTrack,
            NamedKey::NonConvert => KeyCode::NoConvert,
            NamedKey::MediaPlayPause => KeyCode::PlayPause,
            NamedKey::Power => KeyCode::Power,
            NamedKey::MediaSkipBackward => KeyCode::PrevTrack,
            NamedKey::PowerOff => KeyCode::Sleep,
            NamedKey::Tab => KeyCode::Tab,
            NamedKey::AudioVolumeDown => KeyCode::VolumeDown,
            NamedKey::AudioVolumeUp => KeyCode::VolumeUp,
            NamedKey::WakeUp => KeyCode::Wake,
            NamedKey::BrowserBack => KeyCode::WebBack,
            NamedKey::BrowserFavorites => KeyCode::WebFavorites,
            NamedKey::BrowserForward => KeyCode::WebForward,
            NamedKey::BrowserHome => KeyCode::WebHome,
            NamedKey::BrowserRefresh => KeyCode::WebRefresh,
            NamedKey::BrowserSearch => KeyCode::WebSearch,
            NamedKey::BrowserStop => KeyCode::WebStop,
            NamedKey::Copy => KeyCode::Copy,
            NamedKey::Paste => KeyCode::Paste,
            NamedKey::Cut => KeyCode::Cut,
            _ => KeyCode::Unlabeled,
        },
        _ => KeyCode::Unlabeled,
    }
}
