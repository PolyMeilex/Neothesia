warning: function `home` is never used
 --> neothesia-core/src/utils/resources.rs:3:4
  |
3 | fn home() -> Option<PathBuf> {
  |    ^^^^
  |
  = note: `#[warn(dead_code)]` on by default
warning: function `xdg_config` is never used
 --> neothesia-core/src/utils/resources.rs:9:4
  |
9 | fn xdg_config() -> Option<PathBuf> {
  |    ^^^^^^^^^^
   Compiling midi-io v0.1.0 (/Users/runner/work/Neothesia/Neothesia/midi-io)
warning: `neothesia-core` (lib) generated 2 warnings
error[E0277]: the trait bound `iced_core::Element<'_, _, _, _>: From<PianoRange>` is not satisfied
   --> neothesia/src/scene/menu_scene/iced_menu/settings.rs:139:22
    |
139 |           let column = col![
    |  ______________________^
140 | |             output_group,
141 | |             input_group,
142 | |             note_range_group,
143 | |             range,
144 | |             guidelines_group,
145 | |         ]
    | |_________^ the trait `From<PianoRange>` is not implemented for `iced_core::Element<'_, _, _, _>`
    |
    = help: the following other types implement trait `From<T>`:
              `iced_core::Element<'_, Link, Theme, Renderer>` implements `From<Rich<'_, Link, Theme, Renderer>>`
              `iced_core::Element<'_, M, iced_core::Theme, iced_wgpu::Renderer>` implements `From<ActionRow<'_, M>>`
              `iced_core::Element<'_, M, iced_core::Theme, iced_wgpu::Renderer>` implements `From<BarLayout<'_, M>>`
              `iced_core::Element<'_, M, iced_core::Theme, iced_wgpu::Renderer>` implements `From<SegmentButton<M>>`
              `iced_core::Element<'_, M, iced_core::Theme, iced_wgpu::Renderer>` implements `From<TrackCard<'_, M>>`
              `iced_core::Element<'_, M, iced_core::Theme, iced_wgpu::Renderer>` implements `From<neothesia_iced_widgets::Layout<'_, M>>`
              `iced_core::Element<'_, Message, Theme, Renderer>` implements `From<&str>`
              `iced_core::Element<'_, Message, Theme, Renderer>` implements `From<Checkbox<'_, Message, Theme, Renderer>>`
            and 30 others
    = note: this error originates in the macro `col` (in Nightly builds, run with -Z macro-backtrace for more info)
warning: variable does not need to be mutable
   --> neothesia/src/main.rs:256:13
    |
256 |         let mut attributes = winit::window::Window::default_attributes()
    |             ----^^^^^^^^^^
    |             |
    |             help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` on by default
For more information about this error, try `rustc --explain E0277`.
warning: `neothesia` (bin "neothesia") generated 1 warning
error: could not compile `neothesia` (bin "neothesia") due to 1 previous error; 1 warning emitted
Error: Process completed with exit code 101.
