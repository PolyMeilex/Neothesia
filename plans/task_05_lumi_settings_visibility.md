# Task 5: Add Lumi Settings Section (Visible Only When Lumi Connected)

## Description

This task involves adding a dedicated LUMI hardware settings section that only appears when a LUMI keyboard is connected and selected as the input device. Currently, LUMI settings are always visible in the settings menu.

### Objective
Create a dynamic settings section that:
1. Only shows when a LUMI keyboard is detected
2. Is placed after the input selector in the settings UI
3. Contains LUMI-specific controls (brightness, color mode)

## Actionable Checklist

- [ ] **5.1 Analyze Current LUMI Settings Location**
  - Review [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs:149-177) - current LUMI section
  - Review [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs) - input detection logic
  - Review [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - LUMI connection detection

- [ ] **5.2 Implement LUMI Detection**
  - Add method to check if LUMI is connected (check `ctx.output_manager.lumi_connection()`)
  - Store LUMI connection state in UiState or Context
  - Update detection on input selection changes

- [ ] **5.3 Restructure Settings UI**
  - Remove LUMI section from its current location (after Render section)
  - Add new LUMI section AFTER Input selector section
  - Conditionally render based on LUMI detection

- [ ] **5.4 Handle Disconnection**
  - Update UI when LUMI is disconnected
  - Persist settings even when LUMI is not connected
  - Graceful fallback when settings accessed without LUMI

- [ ] **5.5 Add Visual Feedback**
  - Show "No LUMI detected" or similar when not connected
  - Consider adding "connect a LUMI keyboard" prompt
  - Make it clear why the section appears/disappears

## Dependencies and Resources

### Key Files
- [`neothesia/src/scene/menu_scene/settings.rs`](neothesia/src/scene/menu_scene/settings.rs) - Settings UI
- [`neothesia/src/scene/menu_scene/state.rs`](neothesia/src/scene/menu_scene/state.rs) - UI state management
- [`neothesia/src/output_manager/mod.rs`](neothesia/src/output_manager/mod.rs) - LUMI connection handling
- [`neothesia/src/lumi_controller.rs`](neothesia/src/lumi_controller.rs) - LUMI hardware control

### Dependencies
- LUMI SysEx protocol implementation (already exists)
- LUMI brightness/color mode controls (already implemented)

## Potential Challenges

1. **Detection Timing**: LUMI may not be immediately detected; need to handle async detection
2. **Multiple Devices**: What if multiple LUMI devices are connected?
3. **Settings Persistence**: Settings should persist even when LUMI is not connected
4. **User Confusion**: Users may be confused if section appears/disappears

## Success Criteria

- [ ] LUMI settings section ONLY appears when LUMI keyboard is connected
- [ ] Section appears AFTER Input selector in settings order
- [ ] Brightness and color mode controls work correctly
- [ ] UI gracefully handles connection/disconnection
- [ ] Settings are preserved when LUMI is not connected

## UI Layout Proposal

```
Settings
├── Output
│   └── [Output selector]
├── Input  
│   └── [Input selector]
├── LUMI Hardware          ← NEW: Only visible when LUMI detected
│   ├── LED Brightness
│   └── Color Mode
├── Note Range
│   ├── Start
│   └── End
├── Render
│   └── [Toggles...]
└── (existing sections...)
```

## Notes

- Current implementation has LUMI section in a fixed position
- Need to refactor to move it after Input section
- LUMI detection can use existing `lumi_connection()` check

## Progress Tracking

| Date | Progress | Notes |
|------|----------|-------|
|  |  |  |

## Future Updates

- Add per-key RGB color customization
- Add custom LED patterns
- Add LUMI-specific wait mode hints settings
- Support multiple LUMI blocks
