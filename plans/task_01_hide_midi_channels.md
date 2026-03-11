# Task 1: Add Way to "Hide" MIDI Channels from the Waterfall

## Description

This task involves implementing the ability to hide specific MIDI channels from the waterfall visualization while still playing them. Currently, tracks can be hidden in the track selection screen, but there's no way to hide individual channels within a multi-channel MIDI track from the visual display without affecting audio playback.

### Objective
Allow users to selectively hide MIDI channels from the waterfall display while maintaining their audio output. This provides a cleaner visual experience for complex MIDI files with many channels.

## Actionable Checklist

- [ ] **1.1 Analyze Current Track Visibility System**
  - Review [`neothesia/src/scene/menu_scene/tracks.rs`](neothesia/src/scene/menu_scene/tracks.rs) to understand track visibility implementation
  - Review [`neothesia-core/src/render/waterfall/mod.rs`](neothesia-core/src/render/waterfall/mod.rs) to understand how hidden tracks are filtered
  - Understand how `hidden_tracks` vector is passed and used

- [ ] **1.2 Extend Track Configuration for Channel Hiding**
  - Add `hidden_channels: Vec<u8>` field to `TrackConfig` in [`neothesia/src/song.rs`](neothesia/src/song.rs)
  - Update `SongConfig::new()` to initialize empty hidden channels
  - Add methods to toggle channel visibility per track

- [ ] **1.3 Create Channel Visibility UI in Track Selection**
  - Add toggle buttons for channel visibility in track cards
  - Display channel number and current visibility state
  - Ensure UI is responsive and intuitive

- [ ] **1.4 Update Waterfall Renderer**
  - Modify [`neothesia-core/src/render/waterfall/mod.rs`](neothesia-core/src/render/waterfall/mod.rs) to filter hidden channels
  - Update `NoteList::new()` to accept and filter by hidden channels
  - Ensure channel filtering doesn't affect audio playback

- [ ] **1.5 Integrate with Playing Scene**
  - Pass hidden channels to `WaterfallRenderer::new()` in [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs)
  - Ensure hidden channel data is preserved when song is reloaded

## Dependencies and Resources

### Key Files
- [`neothesia/src/song.rs`](neothesia/src/song.rs) - Song and TrackConfig definitions
- [`neothesia/src/scene/menu_scene/tracks.rs`](neothesia/src/scene/menu_scene/tracks.rs) - Track selection UI
- [`neothesia-core/src/render/waterfall/mod.rs`](neothesia-core/src/render/waterfall/mod.rs) - Waterfall rendering logic
- [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) - Playing scene integration

### Dependencies
- No external dependencies required
- Uses existing visibility system patterns

## Potential Challenges

1. **Channel Identification**: MIDI channels (0-15) may not have clear labels; need fallback naming (Channel 1-16)
2. **Performance**: Filtering channels at render time should be efficient; consider caching
3. **Persistence**: Hidden channel preferences should be saved with song config
4. **UI Space**: Track cards have limited space; may need expandable section for channel toggles

## Success Criteria

- [ ] User can hide/show individual MIDI channels within any track
- [ ] Hidden channels are not rendered in the waterfall but still produce audio
- [ ] Channel visibility settings persist during the session
- [ ] UI is intuitive and doesn't clutter the existing track selection interface
- [ ] Implementation follows existing patterns for track visibility

## Notes

- Consider keyboard shortcuts for quick channel hiding
- Could add "Hide All" / "Show All" buttons per track
- May want to add visual indicator on track card showing hidden channel count

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add color-coding for different channel groups
- Support channel grouping (e.g., hide all bass channels)
- Add per-channel volume control
