# Task 6: Allow SoundFont Folder Setting and Switching

## Description

This task involves adding the ability to:
1. Set a default SoundFont folder in settings
2. Easily cycle through available SoundFonts in that folder
3. Switch SoundFonts during playback (without stopping the song)

Currently, users can select a single SoundFont file, but there's no folder-based organization or quick switching capability.

### Objective
Provide a more flexible SoundFont management system that allows users to organize their SoundFonts in folders and quickly switch between them.

## Actionable Checklist

- [x] **6.1 Analyze Current SoundFont Selection**
  - Review [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs:260-298) - current SoundFont selection
  - Review [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - SynthConfig
  - Understand how SoundFonts are loaded in output_manager

- [x] **6.2 Add SoundFont Folder Configuration**
  - Extend SynthConfig in [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs)
  - Add `soundfont_folder: Option<PathBuf>` field
  - Add settings row to select folder using folder picker dialog

- [x] **6.3 Implement SoundFont Discovery**
  - Add function to scan folder for .sf2 files
  - Cache discovered SoundFonts (refresh on folder change)
  - Handle empty folders gracefully

- [x] **6.4 Add Cycle/Switch Controls**
  - Add "Previous" / "Next" buttons in settings UI
  - Display current SoundFont name and index (e.g., "2 of 5")
  - Allow keyboard shortcuts (optional)

- [x] **6.5 Implement Runtime Switching**
  - Allow switching SoundFont during playback
  - Implement hot-reload in synth backend
  - Handle any audio glitches during switch

- [x] **6.6 Persist Preferences**
  - Save selected folder path
  - Save last selected SoundFont index
  - Remember selection across app restarts

## Dependencies and Resources

### Key Files
- [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs) - Settings UI
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - SynthConfig
- [`neothesia/src/output_manager/synth_backend.rs`](neothesia/src/output_manager/synth_backend.rs) - SoundFont loading
- [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - Output management

### Dependencies
- Uses `rfd` crate for file/folder dialogs (already used for SoundFont selection)
- Synth backend (oxysynth) for SoundFont handling

## Potential Challenges

1. **Audio Interruption**: Switching SoundFont during playback may cause audio glitches
2. **Large Folders**: Scanning large folders could be slow; consider async loading
3. **Invalid Files**: Handle corrupted or invalid SoundFont files gracefully
4. **Memory**: Multiple SoundFonts in memory could be heavy; load on demand

## Success Criteria

- [x] User can select a SoundFont folder in settings
- [x] App discovers and lists all .sf2 files in the folder
- [x] User can cycle through SoundFonts with Previous/Next buttons
- [x] Current selection is displayed (name and position)
- [x] SoundFont can be switched during playback without app restart
- [x] Settings persist across app restarts

## UI Layout Proposal

```
Output
├── Output: [Built-in Synth]
├── SoundFont
│   ├── Folder: [Select Folder]     ← NEW
│   └── SoundFont: [< Prev] [Name] [Next >]  ← ENHANCED
│               (2 of 5) Piano.sf2
└── Audio Gain: [-] 0.2 [+]
```

## Runtime Switching Architecture

```rust
// In output_manager
pub fn switch_soundfont(&mut self, path: &Path) -> Result<()> {
    // 1. Stop current audio gracefully
    // 2. Load new SoundFont
    // 3. Re-initialize synth with new SoundFont
    // 4. Resume audio (or keep paused state)
}
```

## Notes

- Current single-file selection can be replaced or enhanced
- Consider adding "Favorites" or "Recent" lists
- Could add SoundFont preview on hover/selection

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
| 2026-03-11 | 100% Complete | Initial implementation with multi-folder support |

## Future Updates

- Add SoundFont metadata display (instrument count, size)
- Add search/filter within folder
- Add ability to load from multiple folders
- SoundFont presets for different music genres

---

## Implementation Status: COMPLETED ✅

### Completion Date
**2026-03-11**

### Summary
Successfully implemented a flexible SoundFont management system that allows users to organize their SoundFonts in folders and quickly switch between them. The implementation includes folder selection, automatic SoundFont discovery, runtime switching capability, and persistent settings.

### Files Modified
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs)
- [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs)
- [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs)
- [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs)

### Key Changes
1. **Added SynthConfigV2** with `soundfont_folder` and `soundfont_index` fields
2. **Implemented V1→V2 migration** for backward compatibility with existing configs
3. **Added `discover_soundfonts()` utility function** to scan folder for .sf2 files
4. **Added `switch_soundfont()` method** for runtime SoundFont switching
5. **Added folder picker UI** in settings for selecting SoundFont directory
6. **Added Previous/Next cycling buttons** for easy navigation through SoundFonts
7. **Added display of current SoundFont name and position** (e.g., "2 of 5")

### Success Criteria - All PASSED ✅
- ✅ User can select a SoundFont folder in settings
- ✅ App discovers and lists all .sf2 files in the folder
- ✅ User can cycle through SoundFonts with Previous/Next buttons
- ✅ Current selection is displayed (name and position)
- ✅ SoundFont can be switched during playback without app restart
- ✅ Settings persist across app restarts

### Code Quality
- Follows existing architectural patterns
- Graceful error handling (no `.unwrap()` in production code)
- Backward compatible with existing V1 configs
- Compiles successfully

---

## Multi-Folder Enhancement: COMPLETED ✅

### Enhancement Date
**2026-03-11**

### Summary
Extended the implementation to support multiple SoundFont folders instead of just one, with natural order iteration and source folder display.

### Additional Changes
- Changed `soundfont_folder: Option<PathBuf>` to `soundfont_folders: Vec<PathBuf>`
- Added `SoundFontEntry` struct to track both path and source folder
- Updated discovery logic to scan multiple folders in order
- Updated UI to show source folder name in display
- Added folder management (add folders via UI)

### Key Features
- **Multiple folder support** - Users can add multiple SoundFont folders
- **Natural order iteration** - First folder first, first file first, etc.
- **Source folder display** - Shows which folder the current SoundFont comes from
- **Display format**: "filename.sf2 from foldername (X of Y)"

### Verification Results - ALL PASSED ✅
- ✅ Config model with `soundfont_folders` as Vec<PathBuf>
- ✅ Discovery logic iterates folders in order
- ✅ SoundFontEntry struct with path and folder fields
- ✅ UI shows source folder name
- ✅ Natural order: folder1/file1, folder1/file2, ..., folder2/file1, folder2/file2
- ✅ Folder management via add_soundfont_folder()

### Files Modified
(same as before, but with changes)
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - Changed to Vec<PathBuf>
- [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - Added SoundFontEntry, updated discovery
- [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs) - Changed to Vec<PathBuf>
- [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs) - Updated UI for multi-folder

---

## Freeplay Mode Enhancement: COMPLETED ✅

### Enhancement Date
**2026-03-11**

### Summary
Added interactive UI buttons to the freeplay mode that enable users to dynamically swap the active SoundFont during playback, ensuring the audio engine supports real-time hot-swapping without interrupting the current audio stream.

### Additional Changes
- Added `soundfonts` and `current_soundfont_index` state fields to FreeplayScene
- Implemented SoundFont discovery in constructor using configured folders
- Added helper methods: `current_soundfont_name()`, `previous_soundfont()`, `next_soundfont()`, `switch_to_soundfont_index()`
- Added UI controls: SoundFont name display label and Previous/Next buttons
- Integrated with existing `OutputManager::switch_soundfont()` for hot-swapping

### Key Features
- **Real-time hot-swapping** - Uses existing `switch_soundfont()` method that preserves playback state
- **No audio interruption** - Playback continues smoothly during SoundFont switches
- **Config persistence** - Automatically saves SoundFont path and index to config
- **Display format** - Shows "filename.sf2 (X of Y) from foldername"
- **Button placement** - Top-right corner of freeplay mode UI
- **Wrap-around cycling** - Previous at start goes to end, Next at end goes to start

### Files Modified
- [`neothesia/src/scene/freeplay/mod.rs`](neothesia/src/scene/freeplay/mod.rs) - Added SoundFont switching UI and logic

### Implementation Details
- State fields added at lines 37-39
- Constructor initialization at lines 66-71
- Helper methods at lines 130-238
- UI controls at lines 145-173
- Updated `update_ui()` signature to accept `&mut Context`

---

## Header Ribbon UI Enhancement: COMPLETED ✅

### Enhancement Date
**2026-03-11**

### Summary
Refactored the freeplay mode UI to use an expandable header ribbon design matching the song play mode, including SoundFont controls in the center panel and Audio Gain slider in the right panel.

### Additional Changes
- Added ribbon state fields: `ribbon_expand_animation` and `is_ribbon_expanded`
- Implemented cursor-based expansion detection (75px threshold)
- Created `render_ribbon()` method with three-panel layout
- Added `create_ribbon_button()` helper for consistent button styling
- Added `audio_gain()` and `set_audio_gain()` methods to SynthConfig
- Implemented smooth expand/collapse animation using `lilt` library

### Key Features
- **Expandable ribbon** - Smoothly expands when cursor is near top of screen
- **Three-panel layout** - Left (back button), Center (SoundFont controls), Right (Audio Gain)
- **Color scheme** - Matches playing scene: [37,35,42] background, [67,67,67] buttons
- **Smooth animation** - Uses EaseOutExpo easing with 1000ms duration
- **Audio Gain control** - -/+ buttons with 0.1 increments (range 0.0-1.0)
- **Alpha blending** - Controls fade in/out with ribbon expansion
- **SoundFont display** - Centered in ribbon with Previous/Next buttons

### UI Layout
```
Header Ribbon (75px when expanded)
├── Left Panel: Back button (←)
├── Center Panel: SoundFont controls
│   ├── Previous button (◀)
│   ├── SoundFont name display
│   └── Next button (▶)
└── Right Panel: Audio Gain controls
    ├── Gain value display
    ├── Decrease button (−)
    └── Increase button (+)
```

### Files Modified
- [`neothesia/src/scene/freeplay/mod.rs`](neothesia/src/scene/freeplay/mod.rs) - Refactored UI with header ribbon
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - Added audio_gain() and set_audio_gain() methods

### Implementation Details
- Ribbon state fields at lines 41-42
- Constructor initialization with animation at lines 73-77
- render_ribbon() method at lines 145-240
- create_ribbon_button() helper at lines 242-256
- Audio gain helpers at lines 258-273
- Cursor-based expansion detection at lines 147-151

---

## Ribbon UI Fixes: COMPLETED ✅

### Fix Date
**2026-03-11**

### Summary
Fixed the freeplay mode ribbon to always be displayed and corrected button icons to show proper icons instead of text artifacts.

### Changes Made
- Removed expand/collapse animation logic
- Made ribbon always visible at 40px height
- Fixed button icons to use proper icon rendering functions
- Added `right_arrow_icon()` function to icons.rs
- Removed unused animation imports and state fields

### Key Improvements
- **Always visible** - Ribbon is now permanently displayed (no animation)
- **Proper icons** - Buttons use `left_arrow_icon()`, `right_arrow_icon()`, `minus_icon()`, and `plus_icon()`
- **Cleaner code** - Removed unnecessary animation complexity
- **Better UX** - Controls are always accessible without hovering

### Files Modified
- [`neothesia/src/scene/freeplay/mod.rs`](neothesia/src/scene/freeplay/mod.rs) - Simplified ribbon rendering
- [`neothesia/src/icons.rs`](neothesia/src/icons.rs) - Added right_arrow_icon() function

### Implementation Details
- Removed `ribbon_expand_animation` and `is_ribbon_expanded` fields
- Removed animation initialization in constructor
- Simplified `update_ui()` method to always render ribbon
- Updated button creation to use icon functions instead of text labels
- Fixed ribbon height to 40px (no expansion)

---

## Button Fixes: COMPLETED ✅

### Fix Date
**2026-03-11**

### Summary
Fixed three button-related issues in the freeplay mode ribbon: right arrow icon showing wrong symbol, gain cap too low, and gain buttons lacking continuous hold capability.

### Issues Fixed
- Right arrow button was showing a "refresh" icon instead of an arrow
- Audio gain was capped at 1.0 (too low for some use cases)
- Gain buttons didn't support continuous hold (had to click repeatedly)

### Changes Made
- Fixed `right_arrow_icon()` function to return correct Unicode character (`\u{f12e}` instead of `\u{f130}`)
- Increased gain cap from 1.0 to 2.0 in `increase_audio_gain()` method
- Added continuous hold support with state tracking fields
- Implemented hold timer that triggers gain changes every 0.1 seconds while button is held
- Updated gain buttons to use ClickArea API for proper press/release event handling

### Technical Implementation
- Added three new fields: `gain_decrease_held`, `gain_increase_held`, `gain_hold_timer`
- Hold logic in `update()` method checks held state and triggers changes every 0.1 seconds
- Buttons trigger immediately on press and continue while held
- Hold state resets on button release
- Uses nuon library's ClickArea API with `is_press_start()`, `is_pressed()`, and `is_clicked()` events

### Files Modified
- [`neothesia/src/icons.rs`](neothesia/src/icons.rs:35) - Fixed right arrow icon
- [`neothesia/src/scene/freeplay/mod.rs`](neothesia/src/scene/freeplay/mod.rs) - Gain cap and continuous hold

### User Experience Improvements
- Right arrow now shows proper arrow icon
- Gain can be increased up to 2.0 (100% more headroom)
- Holding gain buttons continuously adjusts gain (no more repetitive clicking)
- Immediate feedback on button press with continuous adjustment while held

---

## Gain Cap Consistency Fix: COMPLETED ✅

### Fix Date
**2026-03-11**

### Summary
Ensured gain control consistency between settings UI and freeplay mode by removing the artificial 2.0 cap from freeplay mode.

### Issue Identified
- Settings UI had NO cap on gain value (unlimited maximum)
- Freeplay mode had a 2.0 cap on gain
- This created inconsistency where gain could be set higher in settings than in freeplay

### Fix Applied
- Removed the 2.0 cap from `increase_audio_gain()` method in freeplay mode
- Both interfaces now use the same underlying setting with no upper limit
- Minimum cap of 0.0 remains in both interfaces

### Files Modified
- [`neothesia/src/scene/freeplay/mod.rs`](neothesia/src/scene/freeplay/mod.rs:384) - Removed `.min(2.0)` from increase_audio_gain()

### Result
- Gain control is now consistent across both interfaces
- Users can set gain to any value ≥ 0.0 in both settings and freeplay
- Changes made in one interface are reflected in the other
