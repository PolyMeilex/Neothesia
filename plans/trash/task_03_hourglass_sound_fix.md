# Task 3: Fix Sound Playback in Wait/Pause (Hourglass) Mode

## Description

This is a bug fix task. Currently, when "hourglass mode" (wait mode) is enabled, pressing keys does not immediately play the associated sound. Instead, the sound plays only at the end of the notes. This defeats the purpose of the wait mode for learning, as users need immediate audio feedback when they press keys.

### Objective
Fix the sound playback timing in wait mode so that when a user presses a key that matches an expected note, the sound plays immediately rather than waiting until the note would naturally play from the file.

## Actionable Checklist

- [ ] **3.1 Investigate Current Sound Triggering Logic**
  - Review [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) - how sounds are triggered
  - Review [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) - `midi_event` handler
  - Understand the PlayAlong system in midi_player.rs

- [ ] **3.2 Identify the Root Cause**
  - Check if the issue is in the midi_event handler in playing_scene/mod.rs
  - Verify the condition in `ctx.config.wait_mode()` check
  - Check if sound triggering is being blocked by playback logic

- [ ] **3.3 Implement Immediate Sound Playback**
  - Modify [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) `midi_event` handler
  - Add logic to immediately send MIDI NoteOn to output when user presses a required key
  - Ensure the sound plays through the correct audio backend (synth/MIDI)

- [ ] **3.4 Handle Note-Off Events**
  - Ensure NoteOff is also sent immediately when key is released
  - Handle sustain/pedal if applicable

- [ ] **3.5 Test and Verify**
  - Test with both built-in synth and MIDI output
  - Verify sound plays immediately on key press
  - Verify no duplicate sounds when file reaches the same note

## Dependencies and Resources

### Key Files
- [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) - MIDI event handling
- [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) - PlayAlong and sound triggering
- [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - Output connection management
- [`neothesia-core/src/config/model.rs`](neothesia-core/src/config/model.rs) - wait_mode config

### Dependencies
- Existing wait_mode configuration
- PlayAlong system in midi_player.rs
- MIDI output system

## Potential Challenges

1. **Duplicate Sounds**: Need to prevent sound from playing twice (once from user input, once from file)
2. **Timing Windows**: Define acceptable timing window for "correct" key press
3. **Different Outputs**: Handle both synth and MIDI outputs correctly
4. **Latency**: Minimize audio latency for responsive feedback

## Success Criteria

- [ ] Sound plays IMMEDIATELY when user presses a required note in wait mode
- [ ] No duplicate sounds when file playback reaches the same note
- [ ] NoteOff events work correctly (no stuck notes)
- [ ] Works with both built-in synth and MIDI output
- [ ] User gets visual feedback (key lighting) along with audio feedback

## Architecture Note

Based on code analysis in [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs:392-420), the midi_event handler already has wait_mode logic. The fix should enhance this to:
1. Immediately send NoteOn to output_manager when user presses required note
2. Mark note as "user triggered" to prevent file from playing it again

## Notes

- The existing code at lines 392-420 shows wait_mode handling is partially implemented
- The `user_triggered_notes` HashSet in PlayAlong is meant to track this
- Current implementation may have a bug where triggering happens but sound doesn't play

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add different sound timbres for user-triggered notes (to distinguish from file)
- Add velocity sensitivity based on how accurately user hits the note
- Add visual customization for "correct note" feedback
