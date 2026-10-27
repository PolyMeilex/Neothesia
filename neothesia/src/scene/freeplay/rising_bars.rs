use std::collections::HashSet;
use std::time::Duration;

use neothesia_core::Color;
use neothesia_core::render::{QuadInstance, QuadRenderer};

use crate::scene::playing_scene::Keyboard;

/// A single rising bar animation
struct Bar {
    /// The key id this bar is rising from
    key_id: usize,
    /// Current height of the bar (grows while key is pressed, frozen after release)
    height: f32,
    /// How far the bar has moved up after release (0 while key is pressed)
    offset: f32,
    /// Whether the key is still pressed (bar is still growing from the key)
    is_growing: bool,
    /// Color of the bar (captured when the key was pressed)
    color: Color,
}

/// Renders bars that rise up from pressed keys in FreePlay mode.
///
/// When a key is pressed, a bar starts at the key and grows upward.
/// When the key is released, the bar keeps its current length and
/// flies upward as a whole unit until it disappears off the top of the screen.
pub struct RisingBars {
    /// Active bars currently animating
    bars: Vec<Bar>,
    /// Set of key ids that were pressed in the previous frame (for detecting new presses)
    prev_pressed: HashSet<usize>,
}

impl RisingBars {
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            prev_pressed: HashSet::new(),
        }
    }

    /// Update the bars based on currently pressed keys.
    /// Pushes bar quads to the provided quad renderer.
    pub fn update(
        &mut self,
        keyboard: &Keyboard,
        quad_renderer: &mut QuadRenderer,
        delta: Duration,
        animation_speed: f32,
    ) {
        let key_states = keyboard.key_states();
        let keys = &keyboard.layout().keys;
        let keyboard_pos = keyboard.pos();
        // Maximum bar height is the distance from top of screen to keyboard
        let max_height = keyboard_pos.y;
        // Always rise upward, regardless of animation speed direction
        let speed = animation_speed.abs();
        let delta_secs = delta.as_secs_f32();

        // Detect newly pressed keys and create bars for them
        let mut current_pressed = HashSet::new();
        for (key, state) in keys.iter().zip(key_states) {
            let id = key.id();
            if let Some(color) = state.pressed_by_user() {
                current_pressed.insert(id);

                // Create a new bar only on the initial press (not while held)
                if !self.prev_pressed.contains(&id) {
                    self.bars.push(Bar {
                        key_id: id,
                        height: 0.0,
                        offset: 0.0,
                        is_growing: true,
                        color: *color,
                    });
                }
            }
        }
        // Update all bars and render them (uses current_pressed)
        self.bars.retain_mut(|bar| {
            if bar.is_growing {
                // Check if the key is still pressed
                if current_pressed.contains(&bar.key_id) {
                    // Key is still pressed - grow the bar from the key
                    bar.height += delta_secs * speed;
                    bar.height = bar.height.min(max_height);
                } else {
                    // Key was released - freeze the height and start flying up
                    bar.is_growing = false;
                }
            } else {
                // Bar is flying up - move it up (keep its length)
                bar.offset += delta_secs * speed;
            }

            // Remove the bar when it's completely off the top of the screen
            // Bottom of bar is at: keyboard_pos.y - bar.offset
            if keyboard_pos.y - bar.offset < 0.0 {
                return false;
            }

            // Don't render bars that are too small to see
            if bar.height < 2.0 {
                return true;
            }

            // Get key info for positioning
            if let Some(key) = keys.get(bar.key_id) {
                let x = keyboard_pos.x + key.x();
                // Top of bar: keyboard top - height - offset
                let y = keyboard_pos.y - bar.height - bar.offset;
                let width = (key.width() - 1.0).max(1.0);
                // Round only the top corners while growing (attached to keyboard),
                // round all corners when flying freely
                let radius = (key.width() * 0.2).min(bar.height * 0.5);
                let border_radius = if bar.is_growing {
                    [radius, radius, 0.0, 0.0]
                } else {
                    [radius, radius, radius, radius]
                };

                quad_renderer.push(QuadInstance {
                    position: [x, y],
                    size: [width, bar.height],
                    color: bar.color.into_linear_rgba(),
                    // Order: top-left, top-right, bottom-right, bottom-left
                    border_radius,
                    ..Default::default()
                });
            }

            true // Keep this bar
        });

        // Update prev_pressed for next frame (after the closure is done borrowing it)
        self.prev_pressed = current_pressed;
    }

    /// Clear all bars (e.g., when resizing the window)
    pub fn clear(&mut self) {
        self.bars.clear();
        self.prev_pressed.clear();
    }
}

impl Default for RisingBars {
    fn default() -> Self {
        Self::new()
    }
}
