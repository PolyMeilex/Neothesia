# Task 7: Add Play Mode Selection Screen with New Menu Button

## Description

This task involves adding a new "Play Mode" button to the main menu that opens a configuration screen where users can select:
1. **Play Mode**: Watch, Learn, or Play
2. **Channel Selection**: Left hand only, Right hand only, or Both

The existing "Play" button remains unchanged, providing quick access to playback with default settings. The new "Play Mode" button offers advanced configuration for users who want to customize their learning experience.

### Objective
Add a new menu button that navigates to a play mode configuration screen, while preserving all existing menu items and their functionality.

## Actionable Checklist

- [ ] **7.1 Design Play Mode Options**
  - Define three modes:
    - **Watch**: Auto-play through entire song (existing behavior)
    - **Learn**: Wait mode (hourglass) - pauses until user plays correct notes
    - **Play**: User plays along freely (file plays all notes, user can overlay)
  - Determine mode iconography and descriptions

- [ ] **7.2 Design Channel Selection Options**
  - Define options:
    - **Both Hands**: All channels visible and playing
    - **Left Hand Only**: Only show/play left-hand tracks (typically lower notes)
    - **Right Hand Only**: Only show/play right-hand tracks (typically higher notes)
  - Define how hand detection works (MIDI channel, note range, or track assignment)

 - [ ] **7.3 Create New Scene/Page**
   - Add new page enum variant in [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs)
   - Create UI builder function similar to existing pages
   - Design layout with mode buttons and channel toggle

 - [ ] **7.3a Add Play Mode Button to Main Menu**
   - Add new "Play Mode" button to main menu UI
   - Ensure button is visually distinct from "Play" button
   - Position button logically in menu layout
   - Keep all existing menu items unchanged

 - [ ] **7.4 Integrate with Main Menu**
   - Add new "Play Mode" button to main menu UI
   - Keep existing "Play" button unchanged (quick play with defaults)
   - Add navigation from Main/Tracks page to new PlayMode page
   - Pass selected options to PlayingScene

- [ ] **7.5 Apply Settings to Playback**
  - Modify PlayingScene to accept play mode configuration
  - Configure wait_mode based on Learn selection
  - Filter channels based on hand selection

- [ ] **7.6 Handle Back Navigation**
  - Add back button to return to previous screen
  - Handle Escape key to go back
  - Ensure state is clean when navigating away

## Dependencies and Resources

### Key Files
- [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs) - Page enum and state
- [`neothesia/src/scene/menu_scene/mod.rs`](neothesia/src/scene/menu_scene/mod.rs) - Scene UI building and main menu
- [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) - PlayingScene initialization
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - PlaybackConfig

### Dependencies
- Existing UI framework (nuon)
- Wait mode implementation
- Track visibility system

## Potential Challenges

1. **Hand Detection**: Automatically detecting left vs right hand may be inaccurate
2. **Channel Mapping**: Not all MIDI files have clear left/right hand separation
3. **UI Space**: Need to fit new button in main menu without cluttering
4. **Button Distinction**: Users need to understand the difference between "Play" and "Play Mode" buttons
5. **Default Behavior**: Ensure existing "Play" button continues to work as before

## Success Criteria

- [ ] New "Play Mode" button appears in main menu
- [ ] Existing "Play" button remains unchanged and functional
- [ ] Play Mode screen appears when clicking the new button
- [ ] User can select Watch/Learn/Play mode
- [ ] User can select Left/Right/Both hands
- [ ] Selected settings are applied to playback
- [ ] Back navigation works correctly
- [ ] Keyboard shortcuts work (Enter to proceed, Escape to go back)

## UI Layout Proposal

### Main Menu Addition
```
┌─────────────────────────────────────────┐
│            Neothesia Main Menu          │
├─────────────────────────────────────────┤
│                                         │
│            [🎵 Play Song]               │
│        [🎛️ Play Mode ⚙️]               │
│            [⚙️ Settings]                │
│                                         │
└─────────────────────────────────────────┘
```

### Play Mode Selection Screen
```
┌─────────────────────────────────────────┐
│           Select Play Mode              │
├─────────────────────────────────────────┤
│                                         │
│   [🎵 Watch]   [🎓 Learn]   [🎹 Play]   │
│                                         │
│   Watch:  Listen and follow along        │
│   Learn:  Wait for your input           │
│   Play:   Play along freely             │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│   Channels:  [Left] [Right] [Both]      │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│      [← Back]        [Start →]          │
│                                         │
└─────────────────────────────────────────┘
```

## Button Design Considerations

### Visual Distinction
- **Play Button**: Simple, prominent, primary action
  - Icon: ▶️ or 🎵
  - Color: Primary accent color
  - Position: Top or center of menu
  
- **Play Mode Button**: Indicates configuration/options
  - Icon: 🎛️ or ⚙️
  - Color: Secondary or with gear accent
  - Label: "Play Mode" or "Play Mode ⚙️"
  - Position: Below Play button

### Button Behavior
- **Play Button**: Immediate action, starts playback with defaults
- **Play Mode Button**: Navigation action, opens configuration screen

### Accessibility
- Clear visual hierarchy (Play > Play Mode)
- Descriptive tooltips on hover
- Keyboard shortcuts for both buttons
- Screen reader friendly labels

## Play Mode Implementation

```rust
// In PlayingScene::new
pub fn new(ctx: &mut Context, song: Song, play_config: PlayConfig) -> Self {
    // Apply wait mode based on Learn selection
    ctx.config.set_wait_mode(play_config.mode == PlayMode::Learn);
    
    // Filter tracks/channels based on hand selection
    let visible_tracks = filter_tracks_by_hand(&song, play_config.hand);
    
    // ... rest of initialization
}
```

## Main Menu Button Implementation

```rust
// In menu_scene/mod.rs - main menu UI builder
fn build_main_menu(ctx: &mut Context) -> Ui {
    ui::column::column(ctx, [
        // Existing menu items
        build_play_button(ctx),  // Keep unchanged
        build_settings_button(ctx),  // Keep unchanged
        
        // New Play Mode button
        build_play_mode_button(ctx),
        // ... other menu items
    ])
}

fn build_play_mode_button(ctx: &mut Context) -> Ui {
    neo_btn(
        ctx,
        "Play Mode ⚙️",
        || {
            // Navigate to PlayMode page
            ctx.state.set_page(Page::PlayMode);
        },
        // ... button styling
    )
}
```

## Notes

- The "Play" button provides quick access with default settings (Watch mode, Both hands)
- The "Play Mode" button offers advanced configuration for customized learning
- Consider adding "Remember my choice" option in Play Mode screen
- Learn mode requires at least one Human track (see Task 2)
- Button labeling should be clear to distinguish between quick play and configured play
- Could add keyboard shortcut (e.g., 'P' for Play, 'M' for Play Mode)

### Default Play Behavior
When user clicks "Play" button (not "Play Mode"):
- Mode: Watch (auto-play through entire song)
- Channels: Both hands (all visible tracks)
- Wait mode: Disabled
- This preserves existing quick-play workflow

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add "Practice section" selection (loop specific measures)
- Add tempo/speed configuration
- Add metronome toggle
- Add "suggested" mode based on song difficulty
- Consider adding "Quick Play" dropdown from Play Mode button for power users
- Add tooltip to explain difference between Play and Play Mode buttons
