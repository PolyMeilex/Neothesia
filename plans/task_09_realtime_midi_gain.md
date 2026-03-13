# Task 9: Real-Time MIDI Gain Adjustment During Playback

## Description

This task involves implementing a real-time gain adjustment control that allows users to dynamically modify the output volume during active playback. This feature is architecturally distinct from the static input velocity gain currently managed in the settings menu ([`audio_gain`](neothesia-core/src/config/model.rs:103)), which is a persistent configuration value. The new real-time gain control will provide temporary, session-based volume adjustments without affecting the stored configuration.

### Objective

Enable users to adjust MIDI output volume dynamically during playback through an accessible UI control in the playing scene, providing immediate feedback without navigating away from the active performance session.

## Actionable Checklist

- [ ] **9.1 Analyze Current Gain Architecture**
  - Review [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs:100-114) - `SynthConfig` with static `audio_gain`
  - Review [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - Output management and gain application
  - Review [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs:364-371) - Static gain control implementation
  - Understand how `audio_gain` is applied to MIDI events in the synth backend

- [ ] **9.2 Design Real-Time Gain Data Structure**
  - Add `runtime_gain: f32` field to [`PlayingScene`](neothesia/src/scene/playing_scene/mod.rs:17) struct
  - Add `RuntimeGain` wrapper type to encapsulate runtime gain logic
  - Define gain range constraints (e.g., 0.0 to 2.0, with 1.0 as neutral)
  - Add methods for gain adjustment with validation

- [ ] **9.3 Implement Gain Multiplication Logic**
  - Modify output pipeline to multiply static `audio_gain` by `runtime_gain`
  - Ensure gain is applied at the correct stage (before synth or at MIDI event level)
  - Add clamping to prevent excessive volume or silence
  - Handle edge cases (zero gain, negative values)

- [ ] **9.4 Design UI Control in Playing Scene**
  - Add gain control to [`top_bar/mod.rs`](neothesia/src/scene/playing_scene/top_bar/mod.rs:1) - Top bar during playback
  - Design compact slider or spin button interface
  - Add visual feedback showing current gain level (percentage or multiplier)
  - Ensure UI is accessible but doesn't obstruct gameplay

- [ ] **9.5 Implement Keyboard Shortcuts**
  - Add keyboard shortcuts for quick gain adjustment
  - Consider: `[` / `]` for decrease/increase, or `-` / `=`
  - Display shortcut hints in UI or tooltip
  - Ensure shortcuts don't conflict with existing controls

- [ ] **9.6 Add Visual Feedback**
  - Display current gain level in top bar
  - Add visual indicator when gain is modified (e.g., "Gain: 120%")
  - Show toast notification on gain change
  - Consider color coding (red for low, green for normal, yellow for high)

- [ ] **9.7 Handle Persistence and Reset**
  - Ensure `runtime_gain` resets to 1.0 on song change
  - Do NOT persist `runtime_gain` to config file
  - Add "Reset Gain" button or shortcut to return to 1.0
  - Handle scene transitions gracefully

- [ ] **9.8 Integrate with Output Pipeline**
  - Modify [`MidiPlayer`](neothesia/src/scene/playing_scene/midi_player.rs:17) to apply runtime gain
  - Update synth backend to receive combined gain value
  - Ensure gain changes take effect immediately (no lag)
  - Test with both synth and MIDI output backends

## Dependencies and Resources

### Key Files
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - Static `audio_gain` configuration
- [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - Output management and gain application
- [`neothesia/src/output_manager/synth_backend.rs`](neothesia/src/output_manager/synth_backend.rs) - Synth-specific gain handling
- [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) - Playing scene state management
- [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) - MIDI event processing
- [`neothesia/src/scene/playing_scene/top_bar/mod.rs`](neothesia/src/scene/playing_scene/top_bar/mod.rs) - Top bar UI during playback
- [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs) - Reference for existing gain control

### Dependencies
- Existing UI framework (nuon)
- Output manager architecture
- Configuration system (for static gain reference)

## Potential Challenges

1. **Gain Stacking**: Multiple gain points (static + runtime) may cause confusion; need clear documentation
2. **Performance**: Frequent gain recalculation during playback should be efficient
3. **UI Space**: Top bar has limited space; control must be compact
4. **User Confusion**: Users may not understand difference between static and runtime gain
5. **MIDI Output**: Runtime gain may not apply to external MIDI devices (only synth)
6. **Range Limits**: Need appropriate min/max bounds to prevent damage or silence

## Success Criteria

- [ ] Real-time gain control is accessible during playback in the playing scene
- [ ] Gain adjustments take effect immediately without lag
- [ ] Runtime gain is architecturally separate from static `audio_gain` in settings
- [ ] Runtime gain does NOT persist to configuration file
- [ ] Runtime gain resets to 1.0 (neutral) when changing songs or exiting playback
- [ ] UI clearly displays current gain level
- [ ] Keyboard shortcuts work for quick adjustment
- [ ] Implementation follows existing code patterns and conventions

## Architecture Design

### Gain Calculation Formula

```
final_gain = static_audio_gain (from config) × runtime_gain (session-based)
```

Where:
- `static_audio_gain`: Persistent value from settings (default: 0.2)
- `runtime_gain`: Temporary adjustment (default: 1.0, range: 0.0-2.0)
- `final_gain`: Actual gain applied to output

### Data Structure Proposal

```rust
/// Runtime gain multiplier for session-based volume adjustment
/// This is separate from the persistent audio_gain in settings
#[derive(Debug, Clone, Copy)]
pub struct RuntimeGain {
    value: f32, // 1.0 = neutral, 0.0 = silence, 2.0 = double volume
}

impl RuntimeGain {
    pub const NEUTRAL: f32 = 1.0;
    pub const MIN: f32 = 0.0;
    pub const MAX: f32 = 2.0;
    
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(Self::MIN, Self::MAX),
        }
    }
    
    pub fn neutral() -> Self {
        Self { value: Self::NEUTRAL }
    }
    
    pub fn adjust(&mut self, delta: f32) {
        self.value = (self.value + delta).clamp(Self::MIN, Self::MAX);
    }
    
    pub fn reset(&mut self) {
        self.value = Self::NEUTRAL;
    }
    
    pub fn value(&self) -> f32 {
        self.value
    }
    
    pub fn as_percentage(&self) -> u8 {
        (self.value * 100.0) as u8
    }
}
```

### PlayingScene Integration

```rust
pub struct PlayingScene {
    // ... existing fields
    
    /// Runtime gain multiplier for session-based volume control
    /// Separate from persistent config.audio_gain
    runtime_gain: RuntimeGain,
}

impl PlayingScene {
    pub fn new(ctx: &mut Context, song: Song) -> Self {
        Self {
            // ... existing initialization
            runtime_gain: RuntimeGain::neutral(),
        }
    }
    
    pub fn adjust_gain(&mut self, delta: f32) {
        self.runtime_gain.adjust(delta);
        // Notify output manager of gain change
        ctx.output_manager.set_runtime_gain(self.runtime_gain.value());
    }
    
    pub fn reset_gain(&mut self) {
        self.runtime_gain.reset();
        ctx.output_manager.set_runtime_gain(1.0);
    }
    
    pub fn combined_gain(&self) -> f32 {
        ctx.config.audio_gain() * self.runtime_gain.value()
    }
}
```

## UI Layout Proposal

### Top Bar Integration

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Song Title.mid          [Gain: ████░░░░ 100%]  [⚙️] [✕]  │
└─────────────────────────────────────────────────────────────────┘
```

### Gain Control Component

**Option A: Compact Slider**
- Width: 120px
- Height: 20px
- Shows percentage on hover
- Click to drag, or use keyboard shortcuts

**Option B: Spin Buttons (like settings)**
- Label: "Gain: 100%"
- [-] button (decrease by 10%)
- [+] button (increase by 10%)
- More consistent with existing UI

**Option C: Visual Bar with Icons**
- 🔈 icon when low (< 50%)
- 🔉 icon when normal (50-150%)
- 🔊 icon when high (> 150%)
- Click to cycle through presets: 50%, 75%, 100%, 125%, 150%

**Recommendation**: Option B (Spin Buttons) for consistency with settings, with Option C icons as visual feedback

### Full Top Bar Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  [← Back]  Song Title.mid                    [🔉 Gain: 100% [-][+]]  [⚙️]  │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts Proposal

| Shortcut | Action | Description |
|----------|--------|-------------|
| `[` or `-` | Decrease Gain | Reduce runtime gain by 10% |
| `]` or `=` | Increase Gain | Increase runtime gain by 10% |
| `Backspace` or `Delete` | Reset Gain | Reset runtime gain to 100% |
| `M` | Mute Toggle | Set gain to 0% (toggle with previous value) |

## Implementation Notes

### Gain Application Points

The runtime gain should be applied at one of these points:

1. **In OutputManager** (Recommended)
   - Modify `OutputManager::send_note()` or similar methods
   - Apply gain before sending to synth or MIDI backend
   - Centralized location for all output types

2. **In SynthBackend only**
   - Apply gain only for synth output
   - MIDI output would not be affected
   - Simpler but less comprehensive

3. **In MidiPlayer**
   - Apply gain when processing MIDI events
   - Earlier in the pipeline
   - May affect other systems

**Recommendation**: Apply in `OutputManager` for consistency across all output types.

### Toast Notification

When gain is adjusted via keyboard:
```
┌─────────────────────────┐
│   🔉 Gain: 110%         │
└─────────────────────────┘
```
- Auto-dismiss after 2 seconds
- Show in center-bottom of screen
- Use existing toast system ([`toast_manager.rs`](neothesia/src/scene/playing_scene/toast_manager.rs:1))

## Notes

- Static `audio_gain` in settings should be considered the "base volume"
- Runtime gain is a temporary multiplier for the current session
- When both are at default (0.2 and 1.0), final gain = 0.2
- Users who want permanent volume changes should use settings
- Runtime gain is for quick adjustments during practice or performance
- Consider adding "Lock Gain" feature to prevent accidental changes
- MIDI output to external devices may not support runtime gain (document this limitation)

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add per-track gain control (if unified channel control is implemented)
- Add gain presets (e.g., "Quiet Practice", "Performance", "Full Volume")
- Add gain automation (fade in/out on song start/end)
- Add gain curve/smoothing to prevent abrupt changes
- Add visual gain meter (VU meter style)
- Support for saving custom gain profiles per song
- Integration with unified channel control architecture (see [`unified_channel_control_architecture.md`](unified_channel_control_architecture.md))
