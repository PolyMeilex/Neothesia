# Task 4: Continuous Tick on +/- Buttons in Settings

## Description

This task involves adding continuous increment/decrement functionality to the +/- spin buttons in the settings menu. Currently, each click only changes the value by one unit. When users hold the button, it should continuously tick, providing faster adjustment of values.

### Objective
Implement hold-to-repeat functionality for settings spin buttons, similar to how many OS UIs handle slider adjustments.

## Actionable Checklist

- [x] **4.1 Analyze Current Spin Button Implementation**
  - Review [`nuon/src/settings.rs`](nuon/src/settings.rs) - `SettingsRowSpin` and button handling
  - Review [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs) - usage of spin buttons
  - Understand the current click detection mechanism

- [x] **4.2 Design Continuous Tick Mechanism**
  - Add state tracking for button hold duration
  - Define initial delay before repeat starts (e.g., 300ms)
  - Define repeat rate (e.g., every 50ms after initial delay)
  - Handle both + and - buttons independently

- [x] **4.3 Implement in Nuon Library**
  - Modify [`nuon/src/settings.rs`](nuon/src/settings.rs) to add hold detection
  - Add timer/tick mechanism (or expose hold state to consumer)
  - Consider creating a reusable `SpinButton` component

- [x] **4.4 Integrate with Settings UI**
  - Update [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs)
  - The `SettingsRowSpinResult::Plus/Minus` should fire repeatedly while held
  - Ensure update functions handle rapid calls gracefully

- [x] **4.5 Handle Edge Cases**
  - Respect min/max bounds during rapid changes
  - Handle mouse release outside button area
  - Ensure no overflow/underflow in numeric values

## Dependencies and Resources

### Key Files
- [`nuon/src/settings.rs`](nuon/src/settings.rs) - Settings row spin button implementation
- [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs) - Settings UI using spin buttons

### Dependencies
- Uses existing nuon button infrastructure
- No external dependencies

## Potential Challenges

1. **Event Loop Integration**: Need to hook into the update loop for timing
2. **Mouse Up Outside**: Handle case where user drags mouse outside button before releasing
3. **Touch Support**: Should work on touch devices as well
4. **Performance**: Rapid calls shouldn't cause performance issues

## Success Criteria

- [x] Holding +/- button continuously changes value
- [x] Initial delay before repeat starts (prevents accidental rapid changes)
- [x] Consistent repeat rate after initial delay
- [x] Works correctly with all existing settings (brightness, range, etc.)
- [x] No visual glitches during continuous ticking

## Technical Approach

Option 1: Add timing to nuon
```rust
// In nuon/src/settings.rs
pub enum SettingsRowSpinResult {
    Plus,
    Minus,
    PlusHeld,    // NEW: held state
    MinusHeld,   // NEW: held state
    Idle,
}
```

Option 2: Handle in consumer (neothesia)
```rust
// In settings.rs update function
fn tick_held_button(id: &str) -> Option<SpinAction> {
    // track hold state and return repeat actions
}
```

## Notes

- The existing code uses `nuon::SettingsRowSpinResult` which is returned on each build() call
- Need to track hold state between frames in the update loop
- Could add a visual indicator (accelerating ticks) for better UX

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
| 2026-03-11 | ✅ Complete | All requirements implemented and tested successfully |

## Future Updates

- Add acceleration (faster ticks the longer you hold)
- Add haptic feedback on mobile
- Consider adding "jump to default" double-click action
