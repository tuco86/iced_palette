//! The Palette widget - a composable command palette component.
//!
//! # Example
//! ```rust,ignore
//! use iced_palette::{Palette, PaletteState, Command, command};
//!
//! // In your application state:
//! struct App {
//!     palette: PaletteState,
//!     // ...
//! }
//!
//! // Define commands:
//! let commands = vec![
//!     command("save", "Save File")
//!         .shortcut(Shortcut::cmd('s'))
//!         .action(Message::Save),
//! ];
//!
//! // In your view:
//! if self.palette.is_open() {
//!     Palette::new(&self.palette, &commands)
//!         .on_select(|id| Message::CommandSelected(id))
//!         .on_close(Message::PaletteClosed)
//!         .into()
//! }
//! ```

use crate::command::Command;
use crate::search::{filter_commands, FuzzyMatch};
use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, text_input, Column, Row,
};
use iced::{Color, Element, Length, Task, Theme};

/// The ID for the palette's text input widget.
pub const INPUT_ID: &str = "iced_palette_input";

/// State for the command palette.
///
/// Store this in your application state and pass it to `Palette::new()`.
#[derive(Debug, Clone, Default)]
pub struct PaletteState {
    /// Whether the palette is currently open
    open: bool,
    /// Current search query
    query: String,
    /// Currently selected index in the filtered results
    selected_index: usize,
    /// Navigation path for submenus (stack of submenu IDs)
    submenu_path: Vec<String>,
}

impl PaletteState {
    /// Creates a new closed palette state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the palette is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Returns the current search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the currently selected index.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Opens the palette and returns a Task to focus the input.
    pub fn open<Message>(&mut self) -> Task<Message> {
        self.open = true;
        self.query.clear();
        self.selected_index = 0;
        self.submenu_path.clear();
        focus_input()
    }

    /// Closes the palette.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected_index = 0;
        self.submenu_path.clear();
    }

    /// Toggles the palette open/closed and returns a focus Task if opening.
    pub fn toggle<Message>(&mut self) -> Task<Message> {
        if self.open {
            self.close();
            Task::none()
        } else {
            self.open()
        }
    }

    /// Updates the search query.
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0; // Reset selection when query changes
    }

    /// Sets the selected index.
    pub fn set_selected(&mut self, index: usize) {
        self.selected_index = index;
    }

    /// Navigates up in the list with wrapping.
    pub fn navigate_up(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            item_count - 1
        } else {
            self.selected_index - 1
        };
    }

    /// Navigates down in the list with wrapping.
    pub fn navigate_down(&mut self, item_count: usize) {
        if item_count == 0 {
            return;
        }
        self.selected_index = if self.selected_index >= item_count - 1 {
            0
        } else {
            self.selected_index + 1
        };
    }

    /// Enters a submenu.
    pub fn enter_submenu<Message>(&mut self, submenu_id: String) -> Task<Message> {
        self.submenu_path.push(submenu_id);
        self.query.clear();
        self.selected_index = 0;
        focus_input()
    }

    /// Goes back one level in submenu navigation.
    pub fn go_back<Message>(&mut self) -> Task<Message> {
        if self.submenu_path.pop().is_some() {
            self.query.clear();
            self.selected_index = 0;
            focus_input()
        } else {
            Task::none()
        }
    }

    /// Returns the current submenu path.
    pub fn submenu_path(&self) -> &[String] {
        &self.submenu_path
    }
}

/// Returns a Task that focuses the palette input.
pub fn focus_input<Message>() -> Task<Message> {
    iced::widget::operation::focus(iced::widget::Id::new(INPUT_ID))
}

/// Style configuration for the palette.
#[derive(Debug, Clone)]
pub struct PaletteStyle {
    /// Background opacity of the overlay (0.0 - 1.0)
    pub overlay_opacity: f32,
    /// Width of the palette container
    pub width: f32,
    /// Maximum height of the results list
    pub max_height: f32,
    /// Placeholder text for the search input
    pub placeholder: String,
}

impl Default for PaletteStyle {
    fn default() -> Self {
        Self {
            overlay_opacity: 0.5,
            width: 500.0,
            max_height: 400.0,
            placeholder: "Type a command...".to_string(),
        }
    }
}

/// A command palette widget.
///
/// This widget displays a searchable command list with keyboard navigation.
pub struct Palette<'a, Message> {
    state: &'a PaletteState,
    commands: &'a [Command<Message>],
    on_query_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_select: Option<Box<dyn Fn(&'static str) -> Message + 'a>>,
    on_close: Option<Box<dyn Fn() -> Message + 'a>>,
    on_navigate: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    style: PaletteStyle,
}

impl<'a, Message: Clone + 'a> Palette<'a, Message> {
    /// Creates a new Palette widget.
    pub fn new(state: &'a PaletteState, commands: &'a [Command<Message>]) -> Self {
        Self {
            state,
            commands,
            on_query_change: None,
            on_select: None,
            on_close: None,
            on_navigate: None,
            style: PaletteStyle::default(),
        }
    }

    /// Sets the callback for when the search query changes.
    pub fn on_query_change(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_query_change = Some(Box::new(f));
        self
    }

    /// Sets the callback for when a command is selected.
    /// The callback receives the command ID.
    pub fn on_select(mut self, f: impl Fn(&'static str) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Sets the callback for when the palette should close.
    pub fn on_close(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    /// Sets the callback for navigation changes (selection index).
    pub fn on_navigate(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_navigate = Some(Box::new(f));
        self
    }

    /// Sets the style configuration.
    pub fn style(mut self, style: PaletteStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the width of the palette.
    pub fn width(mut self, width: f32) -> Self {
        self.style.width = width;
        self
    }

    /// Sets the overlay opacity.
    pub fn overlay_opacity(mut self, opacity: f32) -> Self {
        self.style.overlay_opacity = opacity;
        self
    }

    /// Sets the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.style.placeholder = placeholder.into();
        self
    }

    /// Builds the palette as an Element.
    pub fn view(self) -> Element<'a, Message> {
        let filtered = filter_commands(&self.state.query, self.commands);
        let selected_index = self.state.selected_index;

        // Build command items with match highlighting
        let command_items: Vec<Element<'a, Message>> = filtered
            .iter()
            .enumerate()
            .map(|(display_index, (original_index, match_result))| {
                let cmd = &self.commands[*original_index];
                let is_selected = display_index == selected_index;

                self.render_command_item(cmd, is_selected, display_index, &match_result)
            })
            .collect();

        let command_list = Column::with_children(command_items).spacing(1);

        // Search input - with or without on_input callback
        let search_input = if let Some(on_change) = self.on_query_change {
            text_input(&self.style.placeholder, &self.state.query)
                .id(INPUT_ID)
                .on_input(on_change)
                .padding([8, 12])
                .size(14)
                .width(Length::Fill)
                .style(|theme: &Theme, _status| input_style(theme))
        } else {
            text_input(&self.style.placeholder, &self.state.query)
                .id(INPUT_ID)
                .padding([8, 12])
                .size(14)
                .width(Length::Fill)
                .style(|theme: &Theme, _status| input_style(theme))
        };

        // Header with search input
        let header = container(search_input).padding([8, 8]);

        // Main palette content
        let palette_content = container(
            column![
                header,
                scrollable(container(command_list).padding([4, 0]).width(Length::Fill))
                    .height(self.style.max_height),
            ]
            .spacing(4)
            .width(self.style.width),
        )
        .style(|theme: &Theme| container_style(theme));

        // Full-screen overlay with click-to-close
        let overlay_opacity = self.style.overlay_opacity;

        if let Some(on_close) = self.on_close {
            mouse_area(
                container(opaque(palette_content))
                    .center(Length::Fill)
                    .style(move |theme: &Theme| overlay_style(theme, overlay_opacity)),
            )
            .on_press(on_close())
            .into()
        } else {
            container(opaque(palette_content))
                .center(Length::Fill)
                .style(move |theme: &Theme| overlay_style(theme, overlay_opacity))
                .into()
        }
    }

    fn render_command_item(
        &self,
        cmd: &Command<Message>,
        is_selected: bool,
        _display_index: usize,
        match_result: &FuzzyMatch,
    ) -> Element<'a, Message> {
        let name = cmd.name.clone();
        let description = cmd.description.clone();
        let shortcut_display = cmd.shortcut.as_ref().map(|s| s.display());

        // Build name with match highlighting
        let name_element: Element<'a, Message> = if !match_result.indices.is_empty() {
            render_highlighted_text(&name, &match_result.indices, is_selected)
        } else {
            text(name.clone()).size(13).into()
        };

        // Left side: name + description
        let left_content: Element<'a, Message> = if let Some(desc) = description {
            row![
                name_element,
                text(desc).size(11).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::text::Style {
                        color: Some(Color::from_rgba(
                            palette.background.base.text.r,
                            palette.background.base.text.g,
                            palette.background.base.text.b,
                            0.5,
                        )),
                    }
                }),
            ]
            .spacing(12)
            .into()
        } else {
            name_element
        };

        // Build full row with optional shortcut on right
        let content: Element<'a, Message> = if let Some(shortcut) = shortcut_display {
            Row::new()
                .push(container(left_content).width(Length::Fill))
                .push(text(shortcut).size(11).style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::text::Style {
                        color: Some(Color::from_rgba(
                            palette.background.base.text.r,
                            palette.background.base.text.g,
                            palette.background.base.text.b,
                            0.4,
                        )),
                    }
                }))
                .align_y(iced::Alignment::Center)
                .width(Length::Fill)
                .into()
        } else {
            Row::new().push(left_content).width(Length::Fill).into()
        };

        // Button with selection handling
        let mut btn = button(content)
            .padding([6, 10])
            .width(Length::Fill)
            .style(move |theme: &Theme, status| item_button_style(theme, is_selected, status));

        if let Some(ref on_select) = self.on_select {
            btn = btn.on_press((on_select)(cmd.id));
        }

        btn.into()
    }
}

/// Renders text with highlighted match characters using Rich text.
fn render_highlighted_text<'a, Message: 'a>(
    text_str: &str,
    indices: &[usize],
    is_selected: bool,
) -> Element<'a, Message> {
    use iced::widget::text::{Rich, Span};

    // If no indices, just return plain text
    if indices.is_empty() {
        return text(text_str.to_string()).size(13).into();
    }

    let chars: Vec<char> = text_str.chars().collect();
    let mut spans: Vec<Span<'a, (), iced::Font>> = Vec::new();
    let mut last_end = 0;

    // Highlight color - blue when not selected, white when selected
    let highlight_color = if is_selected {
        Color::WHITE
    } else {
        Color::from_rgb(0.3, 0.6, 1.0) // Blue highlight
    };

    for &idx in indices {
        // Add non-highlighted segment before this match
        if idx > last_end {
            let segment: String = chars[last_end..idx].iter().collect();
            spans.push(Span::new(segment));
        }
        // Add highlighted character
        if idx < chars.len() {
            let ch: String = chars[idx..idx + 1].iter().collect();
            spans.push(Span::new(ch).color(highlight_color));
        }
        last_end = idx + 1;
    }

    // Add remaining non-highlighted text
    if last_end < chars.len() {
        let segment: String = chars[last_end..].iter().collect();
        spans.push(Span::new(segment));
    }

    Rich::with_spans(spans).size(13).into()
}

// Style functions

fn input_style(theme: &Theme) -> text_input::Style {
    let palette = theme.extended_palette();
    text_input::Style {
        background: iced::Background::Color(palette.background.base.color),
        border: iced::Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: palette.background.weak.text,
        placeholder: Color::from_rgba(
            palette.background.base.text.r,
            palette.background.base.text.g,
            palette.background.base.text.b,
            0.4,
        ),
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    }
}

fn container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(iced::Background::Color(palette.background.weak.color)),
        border: iced::Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    }
}

fn overlay_style(theme: &Theme, opacity: f32) -> container::Style {
    let palette = theme.extended_palette();
    let bg = palette.background.base.color;
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            bg.r, bg.g, bg.b, opacity,
        ))),
        ..container::Style::default()
    }
}

fn item_button_style(
    theme: &Theme,
    is_selected: bool,
    status: button::Status,
) -> button::Style {
    let palette = theme.extended_palette();

    let (background, text_color) = if is_selected {
        (
            Some(iced::Background::Color(palette.primary.base.color)),
            palette.primary.base.text,
        )
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => (
                Some(iced::Background::Color(palette.background.strong.color)),
                palette.background.base.text,
            ),
            _ => (None, palette.background.base.text),
        }
    };

    button::Style {
        background,
        text_color,
        border: iced::Border::default(),
        shadow: iced::Shadow::default(),
        ..Default::default()
    }
}

impl<'a, Message: Clone + 'a> From<Palette<'a, Message>> for Element<'a, Message> {
    fn from(palette: Palette<'a, Message>) -> Self {
        palette.view()
    }
}
