# Architectural Analysis: Unified Channel Control Mechanism

## 1. Current Architecture Summary

### 1.1 PlayerConfig System

The current [`PlayerConfig`](neothesia/src/song.rs:5) enum defines three playback modes applied at the **track level**:

```rust
pub enum PlayerConfig {
    Mute,    // No audio output
    Auto,    // Continuous playback without pausing
    Human,   // Wait state requiring user interaction to proceed
}
```

Each [`TrackConfig`](neothesia/src/song.rs:12) contains:
- `track_id: usize` - Unique track identifier
- `player: PlayerConfig` - Playback mode for the entire track
- `visible: bool` - Track visibility toggle
- `hidden_channels: Vec<u8>` - List of channels to hide visually

### 1.2 Channel Hiding Implementation

Channel hiding was implemented in [Task 1](plans/task_01_hide_midi_channels.md) and provides:
- **Visual filtering only**: Hidden channels are excluded from:
  - Waterfall renderer ([`neothesia-core/src/render/waterfall/mod.rs:17`](neothesia-core/src/render/waterfall/mod.rs:17))
  - Keyboard visual feedback ([`neothesia/src/scene/playing_scene/keyboard.rs:150`](neothesia/src/scene/playing_scene/keyboard.rs:150))
- **Audio unaffected**: Hidden channels still produce sound
- Per-track UI toggles in track cards ([`neothesia/src/scene/menu_scene/tracks.rs:276`](neothesia/src/scene/menu_scene/tracks.rs:276))

### 1.3 Current Separation

| Feature | Level | Audio | Visual |
|---------|-------|-------|--------|
| `PlayerConfig::Mute` | Track | ❌ Blocked | ✅ Visible |
| `PlayerConfig::Auto` | Track | ✅ Play | ✅ Visible |
| `PlayerConfig::Human` | Track | ✅ Play + Wait | ✅ Visible |
| `hidden_channels` | Track | ✅ Play | ❌ Hidden |

### 1.4 Audio Pipeline Flow

In [`MidiPlayer::update()`](neothesia/src/scene/playing_scene/midi_player.rs:54):

```
For each MIDI event:
  1. Get track config
  2. Check PlayerConfig:
     - Mute:  Skip (don't send to output)
     - Auto:  Send to output, don't register with PlayAlong
     - Human: Send to output, register with PlayAlong for wait mode
```

The `PlayAlong` component manages the Human/wait mode logic, tracking required notes and user input.

### 1.5 Files Using PlayerConfig or hidden_channels

| File | Usage |
|------|-------|
| [`neothesia/src/song.rs`](neothesia/src/song.rs) | Definition of `PlayerConfig`, `TrackConfig`, `SongConfig` |
| [`neothesia/src/scene/menu_scene/tracks.rs`](neothesia/src/scene/menu_scene/tracks.rs) | Track card UI with PlayerConfig buttons and channel toggles |
| [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) | Audio playback logic based on PlayerConfig |
| [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) | Passes hidden_channels to renderers |
| [`neothesia/src/scene/playing_scene/keyboard.rs`](neothesia/src/scene/playing_scene/keyboard.rs) | Filters events by hidden_channels |
| [`neothesia-core/src/render/waterfall/mod.rs`](neothesia-core/src/render/waterfall/mod.rs) | Filters notes by hidden_channels |

---

## 2. Proposed New Architecture

### 2.1 Design Goals

1. **Unified mechanism**: Single control per channel combining visibility and playback mode
2. **Four distinct states**: Hidden, Mute, Auto, Human
3. **Channel-level granularity**: Control each MIDI channel independently within a track
4. **Human mode enhancement**: Support parameter combination or synchronization across channels

### 2.2 New Data Structure

Replace the flat `PlayerConfig` + `hidden_channels` with a hierarchical structure:

```rust
/// Unified channel state combining visibility and playback mode
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ChannelState {
    /// Channel excluded from waterfall and keyboard views, produces no sound
    Hidden,
    /// Channel visible in UI elements, but audio is suppressed  
    Mute,
    /// Continuous playback without pausing
    Auto,
    /// Wait state requiring user interaction to proceed
    Human,
}

/// Per-channel configuration within a track
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: u8,           // MIDI channel (0-15)
    pub state: ChannelState,   // Current state
    // Future: Human mode parameters (timing tolerance, etc.)
}

/// Track-level configuration
#[derive(Debug, Clone)]
pub struct TrackConfig {
    pub track_id: usize,
    pub visible: bool,
    pub channels: Vec<ChannelConfig>,  // Per-channel configs
    // Legacy: Keep for migration compatibility, deprecate eventually
    #[deprecated]
    pub player: PlayerConfig,
    #[deprecated]  
    pub hidden_channels: Vec<u8>,
}
```

### 2.3 Default Behavior

On song load, initialize each channel to `Auto`:
- Maintains backward compatibility with current behavior
- `hidden_channels` converts to `ChannelState::Hidden`
- Track-level `player` becomes the default for all channels (migration path)

### 2.4 Human Mode Enhancement

For the requirement "Human state logic should allow parameter combination or synchronization across multiple channels":

```rust
/// Human mode configuration options
#[derive(Debug, Clone, Default)]
pub struct HumanConfig {
    /// Timing tolerance for accepting user input (milliseconds)
    pub timing_tolerance_ms: u32,
    /// Whether this channel waits for user input before advancing
    pub wait_for_input: bool,
    /// Channel ID to synchronize with (None = independent)
    pub sync_with_channel: Option<u8>,
}

/// Extended ChannelConfig with human parameters
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: u8,
    pub state: ChannelState,
    pub human_config: HumanConfig,  // Only relevant when state == Human
}
```

---

## 3. Implementation Plan

### Phase 1: Core Data Structure

- [ ] **1.1** Define `ChannelState` enum in [`neothesia/src/song.rs`](neothesia/src/song.rs)
- [ ] **1.2** Define `ChannelConfig` struct
- [ ] **1.3** Add `channels: Vec<ChannelConfig>` to `TrackConfig`
- [ ] **1.4** Update `SongConfig::new()` to initialize per-channel configs
- [ ] **1.5** Add migration helper to convert `hidden_channels` → `ChannelState::Hidden`
- [ ] **1.6** Add deprecation warnings for old `player` and `hidden_channels` fields

### Phase 2: Audio Pipeline Updates

- [ ] **2.1** Modify [`MidiPlayer::update()`](neothesia/src/scene/playing_scene/midi_player.rs:54) to check channel-level state
- [ ] **2.2** Implement per-channel Mute logic (skip output, but allow visual)
- [ ] **2.3** Implement per-channel Auto vs Human logic
- [ ] **2.4** Handle Human mode synchronization between channels
- [ ] **2.5** Update `PlayAlong` to support channel-specific wait states

### Phase 3: Visual Rendering Updates

- [ ] **3.1** Modify [`WaterfallRenderer`](neothesia-core/src/render/waterfall/mod.rs) to use `ChannelState::Hidden`
- [ ] **3.2** Update [`Keyboard`](neothesia/src/scene/playing_scene/keyboard.rs) to filter by channel state
- [ ] **3.3** Update [`PlayingScene::new()`](neothesia/src/scene/playing_scene/mod.rs:61) to pass new config structure

### Phase 4: UI Implementation

- [ ] **4.1** Replace separate PlayerConfig buttons with unified channel state toggle
- [ ] **4.2** Design compact 4-state toggle for each channel button
- [ ] **4.3** Add visual indicators for each state (colors/icons)
- [ ] **4.4** Handle space constraints in track card (344x126 pixels)
- [ ] **4.5** Add state cycling or dropdown for state selection

### Phase 5: Testing & Polish

- [ ] **5.1** Test all four states work correctly
- [ ] **5.2** Test Human mode synchronization between channels
- [ ] **5.3** Verify backward compatibility with old configurations
- [ ] **5.4** Update documentation

---

## 4. UI Design Recommendations

### 4.1 Track Card Layout (344x126 pixels)

Current layout has:
- Visibility toggle: 40x40px icon button (top-left)
- Labels: Title + subtitle (next to visibility toggle)
- Channel toggles: Row of 18x18px buttons below labels
- Player config: 3 buttons (Mute/Auto/Human) at bottom

**Proposed unified design:**

```
┌──────────────────────────────────────────┐
│ [👁] Track Title              42 Notes │
│      Channel: [1][2][3][4]              │
│      ┌───┬───┬───┬───┐                  │
│      │ H │ M │ A │ U │                  │
│      └───┴───┴───┴───┘                  │
└──────────────────────────────────────────┘
```

Where:
- **H** = Hidden (red, excluded from visual + audio)
- **M** = Mute (gray, visible but no audio)
- **A** = Auto (green, continuous playback)
- **U** = Human/User (blue, wait for input)

### 4.2 State Toggle Interaction

**Option A: Cycle on Click**
- Click channel button → cycle through states: Auto → Human → Mute → Hidden → Auto
- Simple but may be frustrating for quick changes

**Option B: Right-Click Context Menu**
- Left-click: Toggle visibility (show/hide)
- Right-click: Open state selection menu
- More complex but more flexible

**Option C: State-Specific Colors**
- Each button color represents current state
- Click to open dropdown with all 4 options
- Visual clarity, slightly more space needed

**Recommendation**: Option C with dropdown, fallback to cycling on double-click

### 4.3 Visual State Indicators

| State | Color | Icon | Description |
|-------|-------|------|-------------|
| Hidden | 🔴 Red (#B45050) | ○ | Not rendered, no sound |
| Mute | ⚪ Gray (#4A4458) | 🔇 | Rendered, no sound |
| Auto | 🟢 Green (#50B470) | ▶️ | Rendered, auto-play |
| Human | 🔵 Blue (#5070B4) | ⏳ | Rendered, wait for user |

### 4.4 Human Mode Synchronization UI

For channels that should synchronize (e.g., left+right hand wait together):

```
Channel: [1(H)][2(H)][3(A)][4(A)]
                  ↑
            Sync indicator
```

- Tap and hold on Human state → opens sync options
- "Sync with channel X" option
- Visual link indicator between synchronized channels

---

## 5. Questions and Clarifications

### 5.1 Ambiguities to Resolve

1. **Default state for new channels**: Should new songs initialize all channels to Auto, or inherit from track-level PlayerConfig?

2. **Migration from existing configs**: 
   - How to handle songs saved with old `PlayerConfig` + `hidden_channels`?
   - Option A: Auto-migrate on load (hidden → Hidden, Mute → Mute, others → Auto)
   - Option B: Require explicit user migration
   - Option C: Keep legacy fields, new UI operates on new fields

3. **Human mode synchronization**: 
   - Should synchronization wait for ANY channel or ALL channels in sync group?
   - How to handle timing differences between synchronized channels?

4. **Track vs Channel defaults**:
   - Should there be a "track default" that applies to all channels?
   - Or should each channel always be explicitly configured?

### 5.2 Trade-offs to Consider

| Approach | Pros | Cons |
|----------|------|------|
| Per-channel only | Full flexibility | More UI complexity |
| Track default + override | Simpler UI | Less granular |
| Hybrid (default + per-channel) | Balanced | More code complexity |

### 5.3 Recommendations

1. **Start simple**: Implement per-channel with Auto as default, then add track defaults if needed

2. **Preserve backward compatibility**: Keep old fields but mark deprecated, auto-migrate on load

3. **Human sync**: Start with independent channels, add basic "wait for all" synchronization first

4. **UI**: Use color-coded buttons with dropdown for state selection to balance clarity and space

---

## 6. Summary

The current architecture separates track-level playback mode (`PlayerConfig`) from channel-level visibility (`hidden_channels`). The refactor merges these into a unified per-channel mechanism with four states:

1. **Hidden**: Combines visual hiding + audio suppression
2. **Mute**: Visual display + audio suppression  
3. **Auto**: Visual display + continuous playback
4. **Human**: Visual display + wait-for-input playback

The implementation requires changes to:
- Data structures in `song.rs`
- Audio pipeline in `midi_player.rs`
- Visual rendering in `waterfall/mod.rs` and `keyboard.rs`
- UI in `tracks.rs`

A phased approach allows incremental implementation and testing while maintaining backward compatibility.
