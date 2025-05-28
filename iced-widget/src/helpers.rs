//! Helper functions to create pure widgets.
use crate::button::{self, Button};
use crate::checkbox::{self, Checkbox};
use crate::container::{self, Container};
use crate::core;
use crate::core::{Element, Length};
use crate::overlay;
use crate::pick_list::{self, PickList};
use crate::radio::{self, Radio};
use crate::scrollable::{self, Scrollable};
use crate::text::{self, Text};
use crate::toggler::{self, Toggler};
use crate::tooltip::{self, Tooltip};
use crate::{Column, MouseArea, Row, Space, Stack};

use std::borrow::Borrow;

/// Creates a [`Column`] with the given children.
///
/// Columns distribute their children vertically.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{button, column};
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     column![
///         "I am on top!",
///         button("I am in the center!"),
///         "I am below.",
///     ].into()
/// }
/// ```
#[macro_export]
macro_rules! column {
    () => (
        $crate::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Column::with_children([$($crate::core::Element::from($x)),+])
    );
}

/// Creates a [`Row`] with the given children.
///
/// Rows distribute their children horizontally.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{button, row};
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     row![
///         "I am to the left!",
///         button("I am in the middle!"),
///         "I am to the right!",
///     ].into()
/// }
/// ```
#[macro_export]
macro_rules! row {
    () => (
        $crate::Row::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Row::with_children([$($crate::core::Element::from($x)),+])
    );
}

/// Creates a [`Stack`] with the given children.
///
/// [`Stack`]: crate::Stack
#[macro_export]
macro_rules! stack {
    () => (
        $crate::Stack::new()
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Stack::with_children([$($crate::core::Element::from($x)),+])
    );
}

/// Creates a new [`Text`] widget with the provided content.
///
/// [`Text`]: core::widget::Text
///
/// This macro uses the same syntax as [`format!`], but creates a new [`Text`] widget instead.
///
/// See [the formatting documentation in `std::fmt`](std::fmt)
/// for details of the macro argument syntax.
///
/// # Examples
///
/// ```no_run
/// # mod iced {
/// #     pub mod widget {
/// #         macro_rules! text {
/// #           ($($arg:tt)*) => {unimplemented!()}
/// #         }
/// #         pub(crate) use text;
/// #     }
/// # }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::core::Theme, ()>;
/// use iced::widget::text;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(_state: &State) -> Element<Message> {
///     let simple = text!("Hello, world!");
///
///     let keyword = text!("Hello, {}", "world!");
///
///     let planet = "Earth";
///     let local_variable = text!("Hello, {planet}!");
///     // ...
///     # unimplemented!()
/// }
/// ```
#[macro_export]
macro_rules! text {
    ($($arg:tt)*) => {
        $crate::Text::new(format!($($arg)*))
    };
}

/// Creates a new [`Container`] with the provided content.
///
/// Containers let you align a widget inside their boundaries.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::container;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     container("This text is centered inside a rounded box!")
///         .padding(10)
///         .center(800)
///         .style(container::rounded_box)
///         .into()
/// }
/// ```
pub fn container<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    Container::new(content)
}

/// Creates a new [`Container`] that fills all the available space
/// and centers its contents inside.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let center = container("Center!").center(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn center<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content).center(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// horizontally and centers its contents inside.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let center_x = container("Horizontal Center!").center_x(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn center_x<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content).center_x(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// vertically and centers its contents inside.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let center_y = container("Vertical Center!").center_y(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn center_y<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content).center_y(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// horizontally and right-aligns its contents inside.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let right = container("Right!").align_right(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn right<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content).align_right(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// and aligns its contents inside to the right center.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let right_center = container("Bottom Center!").align_right(Fill).center_y(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn right_center<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content)
        .align_right(Length::Fill)
        .center_y(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// vertically and bottom-aligns its contents inside.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let bottom = container("Bottom!").align_bottom(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn bottom<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content).align_bottom(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// and aligns its contents inside to the bottom center.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let bottom_center = container("Bottom Center!").center_x(Fill).align_bottom(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn bottom_center<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content)
        .center_x(Length::Fill)
        .align_bottom(Length::Fill)
}

/// Creates a new [`Container`] that fills all the available space
/// and aligns its contents inside to the bottom right corner.
///
/// This is equivalent to:
/// ```rust,no_run
/// # use iced_widget::core::Length::Fill;
/// # use iced_widget::Container;
/// # fn container<A>(x: A) -> Container<'static, ()> { unreachable!() }
/// let bottom_right = container("Bottom!").align_right(Fill).align_bottom(Fill);
/// ```
///
/// [`Container`]: crate::Container
pub fn bottom_right<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::Renderer,
{
    container(content)
        .align_right(Length::Fill)
        .align_bottom(Length::Fill)
}

/// Creates a new [`Column`] with the given children.
///
/// Columns distribute their children vertically.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, text};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     column((0..5).map(|i| text!("Item {i}").into())).into()
/// }
/// ```
pub fn column<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Column<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Column::with_children(children)
}

/// Creates a new [`Row`] from an iterator.
///
/// Rows distribute their children horizontally.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{row, text};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     row((0..5).map(|i| text!("Item {i}").into())).into()
/// }
/// ```
pub fn row<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Row<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Row::with_children(children)
}

/// Creates a new [`Stack`] with the given children.
///
/// [`Stack`]: crate::Stack
pub fn stack<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> Stack<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    Stack::with_children(children)
}

/// Creates a new [`Scrollable`] with the provided content.
///
/// Scrollables let users navigate an endless amount of content with a scrollbar.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, scrollable, vertical_space};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     scrollable(column![
///         "Scroll me!",
///         vertical_space().height(3000),
///         "You did it!",
///     ]).into()
/// }
/// ```
pub fn scrollable<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Scrollable<'a, Message, Theme, Renderer>
where
    Theme: scrollable::Catalog + 'a,
    Renderer: core::Renderer,
{
    Scrollable::new(content)
}

/// Creates a new [`Button`] with the provided content.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::button;
///
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     button("Press me!").on_press(Message::ButtonPressed).into()
/// }
/// ```
pub fn button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Theme: button::Catalog + 'a,
    Renderer: core::Renderer,
{
    Button::new(content)
}

/// Creates a new [`Tooltip`] for the provided content with the given
/// [`Element`] and [`tooltip::Position`].
///
/// Tooltips display a hint of information over some element when hovered.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{container, tooltip};
///
/// enum Message {
///     // ...
/// }
///
/// fn view(_state: &State) -> Element<'_, Message> {
///     tooltip(
///         "Hover me to display the tooltip!",
///         container("This is the tooltip contents!")
///             .padding(10)
///             .style(container::rounded_box),
///         tooltip::Position::Bottom,
///     ).into()
/// }
/// ```
pub fn tooltip<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    tooltip: impl Into<Element<'a, Message, Theme, Renderer>>,
    position: tooltip::Position,
) -> crate::Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Tooltip::new(content, tooltip, position)
}

/// Creates a new [`Text`] widget with the provided content.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::core::Theme, ()>;
/// use iced::widget::text;
/// use iced::color;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     text("Hello, this is iced!")
///         .size(20)
///         .color(color!(0x0000ff))
///         .into()
/// }
/// ```
pub fn text<'a, Theme, Renderer>(text: impl text::IntoFragment<'a>) -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Text::new(text)
}

/// Creates a new [`Span`] of text with the provided content.
///
/// A [`Span`] is a fragment of some [`Rich`] text.
///
/// [`Span`]: text::Span
/// [`Rich`]: text::Rich
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::*; }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::font;
/// use iced::widget::{rich_text, span};
/// use iced::{color, never, Font};
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     rich_text![
///         span("I am red!").color(color!(0xff0000)),
///         " ",
///         span("And I am bold!").font(Font { weight: font::Weight::Bold, ..Font::default() }),
///     ]
///     .on_link_click(never)
///     .size(20)
///     .into()
/// }
/// ```
pub fn span<'a, Link, Font>(text: impl text::IntoFragment<'a>) -> text::Span<'a, Link, Font> {
    text::Span::new(text)
}

/// Creates a new [`Checkbox`].
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::checkbox;
///
/// struct State {
///    is_checked: bool,
/// }
///
/// enum Message {
///     CheckboxToggled(bool),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     checkbox("Toggle me!", state.is_checked)
///         .on_toggle(Message::CheckboxToggled)
///         .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::CheckboxToggled(is_checked) => {
///             state.is_checked = is_checked;
///         }
///     }
/// }
/// ```
/// ![Checkbox drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/checkbox.png?raw=true)
pub fn checkbox<'a, Message, Theme, Renderer>(
    label: impl Into<String>,
    is_checked: bool,
) -> Checkbox<'a, Message, Theme, Renderer>
where
    Theme: checkbox::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Checkbox::new(label, is_checked)
}

/// Creates a new [`Radio`].
///
/// Radio buttons let users choose a single option from a bunch of options.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::{column, radio};
///
/// struct State {
///    selection: Option<Choice>,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// enum Message {
///     RadioSelected(Choice),
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Choice {
///     A,
///     B,
///     C,
///     All,
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     let a = radio(
///         "A",
///         Choice::A,
///         state.selection,
///         Message::RadioSelected,
///     );
///
///     let b = radio(
///         "B",
///         Choice::B,
///         state.selection,
///         Message::RadioSelected,
///     );
///
///     let c = radio(
///         "C",
///         Choice::C,
///         state.selection,
///         Message::RadioSelected,
///     );
///
///     let all = radio(
///         "All of the above",
///         Choice::All,
///         state.selection,
///         Message::RadioSelected
///     );
///
///     column![a, b, c, all].into()
/// }
/// ```
pub fn radio<'a, Message, Theme, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Radio<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: radio::Catalog + 'a,
    Renderer: core::text::Renderer,
    V: Copy + Eq,
{
    Radio::new(label, value, selected, on_click)
}

/// Creates a new [`Toggler`].
///
/// Togglers let users make binary choices by toggling a switch.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::toggler;
///
/// struct State {
///    is_checked: bool,
/// }
///
/// enum Message {
///     TogglerToggled(bool),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     toggler(state.is_checked)
///         .label("Toggle me!")
///         .on_toggle(Message::TogglerToggled)
///         .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::TogglerToggled(is_checked) => {
///             state.is_checked = is_checked;
///         }
///     }
/// }
/// ```
pub fn toggler<'a, Message, Theme, Renderer>(
    is_checked: bool,
) -> Toggler<'a, Message, Theme, Renderer>
where
    Theme: toggler::Catalog + 'a,
    Renderer: core::text::Renderer,
{
    Toggler::new(is_checked)
}

/// Creates a new [`PickList`].
///
/// Pick lists display a dropdown list of selectable options.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::pick_list;
///
/// struct State {
///    favorite: Option<Fruit>,
/// }
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// enum Fruit {
///     Apple,
///     Orange,
///     Strawberry,
///     Tomato,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     FruitSelected(Fruit),
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     let fruits = [
///         Fruit::Apple,
///         Fruit::Orange,
///         Fruit::Strawberry,
///         Fruit::Tomato,
///     ];
///
///     pick_list(
///         fruits,
///         state.favorite,
///         Message::FruitSelected,
///     )
///     .placeholder("Select your favorite fruit...")
///     .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::FruitSelected(fruit) => {
///             state.favorite = Some(fruit);
///         }
///     }
/// }
///
/// impl std::fmt::Display for Fruit {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         f.write_str(match self {
///             Self::Apple => "Apple",
///             Self::Orange => "Orange",
///             Self::Strawberry => "Strawberry",
///             Self::Tomato => "Tomato",
///         })
///     }
/// }
/// ```
pub fn pick_list<'a, T, L, V, Message, Theme, Renderer>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone,
    Theme: pick_list::Catalog + overlay::menu::Catalog,
    Renderer: core::text::Renderer,
{
    PickList::new(options, selected, on_selected)
}

/// Creates a new [`Space`] widget that fills the available
/// horizontal space.
///
/// This can be useful to separate widgets in a [`Row`].
pub fn horizontal_space() -> Space {
    Space::with_width(Length::Fill)
}

/// Creates a new [`Space`] widget that fills the available
/// vertical space.
///
/// This can be useful to separate widgets in a [`Column`].
pub fn vertical_space() -> Space {
    Space::with_height(Length::Fill)
}

/// Creates a new [`Image`].
///
/// Images display raster graphics in different formats (PNG, JPG, etc.).
///
/// [`Image`]: crate::Image
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::image;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     image("ferris.png").into()
/// }
/// ```
/// <img src="https://github.com/iced-rs/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
pub fn image<Handle>(handle: impl Into<Handle>) -> crate::Image<Handle> {
    crate::Image::new(handle.into())
}

/// Creates a new [`MouseArea`].
pub fn mouse_area<'a, Message, Theme, Renderer>(
    widget: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> MouseArea<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    MouseArea::new(widget)
}
