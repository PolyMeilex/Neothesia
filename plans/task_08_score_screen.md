# Task 8: Add Score Screen at End of Song

## Description

This task involves adding a score/results screen that appears when a song finishes playing. The screen should display performance metrics including accuracy, timing information, and other relevant statistics.

### Objective
Provide users with feedback on their performance after completing a song, including accuracy percentage, timing analysis, and other useful metrics.

## Actionable Checklist

- [ ] **8.1 Analyze Existing Stats Tracking**
  - Review [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs:233-270) - `PlayerStats` struct
  - Understand what stats are currently tracked (wrong_notes, early, late)
  - Review PlayAlong statistics collection

- [ ] **8.2 Define Score Metrics**
  - Determine what metrics to display:
    - Accuracy percentage (correct notes / total notes)
    - Timing breakdown (too early, on time, too late)
    - Total notes played
    - Notes missed
    - Streak/best streak
  - Define grading scale (S/A/B/C/D/F or percentage-based)

- [ ] **8.3 Create Score Screen UI**
  - Add new scene/page for score display
  - Design layout showing all metrics
  - Add visual feedback (colors, icons)
  - Include "Retry" and "Continue" buttons

- [ ] **8.4 Capture Stats at Song End**
  - Modify [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs:335-339) - song finish handling
  - Extract stats from PlayAlong before scene transition
  - Pass stats to score screen

- [ ] **8.5 Implement Navigation**
  - Connect score screen to main flow
  - Handle "Play Again" - restart song
  - Handle "Continue" - return to menu
  - Handle Escape key to skip to menu

- [ ] **8.6 Handle Edge Cases**
  - What if user exits mid-song?
  - What if song has no playable notes?
  - Handle Watch mode (no user input = no score?)

## Dependencies and Resources

### Key Files
- [`neothesia/src/scene/playing_scene/midi_player.rs`](neothesia/src/scene/playing_scene/midi_player.rs) - PlayerStats
- [`neothesia/src/scene/playing_scene/mod.rs`](neothesia/src/scene/playing_scene/mod.rs) - Song end handling
- [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs) - Page navigation
- [`neothesia/src/main.rs`](neothesia/src/main.rs) - NeothesiaEvent flow

### Dependencies
- Existing stats tracking in PlayAlong
- UI framework (nuon)
- Scene management system

## Potential Challenges

1. **Score in Different Modes**: Watch mode shouldn't have a score; Learn vs Play may differ
2. **Stat Persistence**: Stats are lost when PlayingScene drops; need to capture before
3. **Empty Stats**: Handle case where no user input occurred
4. **Visual Design**: Make it visually appealing but not overly complex

## Success Criteria

- [ ] Score screen appears when song finishes (in Learn/Play mode)
- [ ] Displays accuracy percentage
- [ ] Shows timing breakdown (early/on-time/late)
- [ ] Shows total notes and missed notes
- [ ] "Play Again" button restarts the song
- [ ] "Continue" button returns to menu
- [ ] Gracefully handles edge cases

## Score Screen UI Proposal

```
┌─────────────────────────────────────────┐
│           Song Complete!                │
├─────────────────────────────────────────┤
│                                         │
│              ★★★☆☆                       │
│            Grade: B                      │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│   Accuracy:     87%  (156 / 179)       │
│   Perfect:      102                     │
│   Good:          42                      │
│   Missed:        21                     │
│   Too Early:     10                     │
│   Too Late:       4                     │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│      [↺ Replay]      [Continue →]       │
│                                         │
└─────────────────────────────────────────┘
```

## Stats Extraction

```rust
// In PlayingScene before transition
fn get_score(&self) -> ScoreData {
    let stats = &self.player.play_along().stats;
    ScoreData {
        total_notes: stats.total_notes(),
        accuracy: stats.timing_accuracy(),
        perfect: stats.perfect_count(),
        good: stats.good_count(),
        missed: stats.wrong_notes,
        // ...
    }
}
```

## Grading Scale Suggestion

| Grade | Accuracy |
|-------|----------|
| S     | 95%+     |
| A     | 85-94%   |
| B     | 70-84%   |
| C     | 55-69%   |
| D     | 40-54%   |
| F     | <40%     |

## Notes

- Only show score for Learn/Play modes; skip for Watch mode
- Consider adding high score persistence (local storage)
- Could add share functionality (export score)

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add per-hand scoring
- Add "best score" tracking per song
- Add progress charts over time
- Add achievement badges
- Support for scoreboards
