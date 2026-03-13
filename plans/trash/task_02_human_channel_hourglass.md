# Task 2: Ensure One Channel is Set to "Human" in Hourglass Mode

## Description

This task involves ensuring that when "hourglass mode" (wait mode) is enabled, at least one track/channel is automatically set to "Human" player mode. This allows the user to play along with the song while the waterfall pauses waiting for user input.

### Objective
Automatically configure one channel to "Human" when wait mode is enabled, ensuring the hourglass functionality works as intended for interactive learning.

## Actionable Checklist

- [ ] **2.1 Analyze Current Player Config System**
  - Review [`neothesia/src/song.rs`](neothesia/src/song.rs) - `PlayerConfig` enum (Mute, Auto, Human)
  - Review [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) - how Human mode triggers play_along
  - Understand how wait mode interacts with Human tracks

- [ ] **2.2 Implement Auto Human Channel Assignment**
  - Modify [`neothesia/src/scene/menu_scene/tracks.rs`](neothesia/src/scene/menu_scene/tracks.rs) - when saving track config
  - Check if wait mode is enabled in config before assigning Human
  - Find first non-drum track and set to Human automatically
  - Add logic to ensure only one track is set to Human (or allow multiple)

- [ ] **2.3 Add UI Indication**
  - Display which track is set to Human in the tracks selection UI
  - Show "Auto-assigned for wait mode" indicator when automatic assignment occurs
  - Allow user to manually override the assignment

- [ ] **2.4 Handle Edge Cases**
  - What if all tracks are set to Mute? 
  - What if there are no non-drum tracks?
  - Handle song changes (re-analyze when new song loads)

## Dependencies and Resources

### Key Files
- [`neothesia/src/song.rs`](neothesia/src/song.rs) - `PlayerConfig`, `TrackConfig`, `SongConfig`
- [`neothesia/src/scene/menu_scene/tracks.rs`](neothesia/src/scene/menu_scene/tracks.rs) - Track selection UI
- [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) - PlayAlong logic
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - PlaybackConfig (wait_mode)

### Dependencies
- Wait mode configuration already exists in config
- Track player modes already implemented

## Potential Challenges

1. **User Intent**: Users may want specific tracks as Human; need to respect manual overrides
2. **Multiple Human Tracks**: Should multiple tracks be Human? Need to define behavior
3. **Drum Tracks**: Should drum tracks ever be Human? Typically no
4. **Persistence**: Should the auto-assignment be remembered per song?

## Success Criteria

- [ ] When wait mode is enabled, at least one track is automatically set to Human
- [ ] The Human track is visually indicated in the UI
- [ ] User can manually change which track is Human
- [ ] System handles edge cases gracefully (all muted, no melodic tracks, etc.)
- [ ] Behavior is predictable and doesn't surprise users

## Notes

- Consider adding a "Suggest Human Track" button instead of auto-assigning
- Could add a dropdown to select which track is the "learning track"
- The hourglass mode relies on PlayAlong system in midi_player.rs

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add smart defaults based on instrument detection (e.g., prefer piano/guitar tracks)
- Support for multiple simultaneous Human tracks
- Different wait mode behaviors (e.g., strict vs. lenient timing)
