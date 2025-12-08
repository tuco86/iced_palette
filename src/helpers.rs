//! Helper functions for simple command palette integration.

use crate::command::Command;
use crate::search::filter_commands;
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, text, text_input, Column, Row,
};
use iced::{Color, Element, Length, Task, Theme};

/// The widget ID for the command palette's text input.
/// Use this with `focus_input()` to focus the input when opening the palette.
pub const INPUT_ID: &str = "iced_palette_input";

/// Returns a Task that focuses the command palette input.
/// Call this from your update function when opening the palette.
///
/// # Example
/// ```rust,ignore
/// ApplicationMessage::ToggleCommandPalette => {
///     self.command_palette_open = !self.command_palette_open;
///     if self.command_palette_open {
///         return iced_palette::focus_input();
///     }
///     Task::none()
/// }
/// ```
pub fn focus_input<Message>() -> Task<Message> {
    iced::widget::operation::focus(iced::widget::Id::new(INPUT_ID))
}

/// Configuration for the command palette appearance.
#[derive(Debug, Clone)]
pub struct PaletteConfig {
    /// Opacity of the background overlay (0.0 - 1.0). Default: 0.1
    pub background_opacity: f32,
    /// Width of the palette in pixels. Default: 500
    pub width: f32,
    /// Maximum height of the command list. Default: 300
    pub max_height: f32,
    /// Placeholder text for the search input. Default: "Type to search..."
    pub placeholder: String,
}

impl Default for PaletteConfig {
    fn default() -> Self {
        Self {
            background_opacity: 0.1,
            width: 500.0,
            max_height: 300.0,
            placeholder: "Type to search...".to_string(),
        }
    }
}

/// Renders a command palette overlay with search input and default configuration.
pub fn command_palette<'a, Message: Clone + 'a>(
    query: &str,
    commands: &[Command<Message>],
    selected_index: usize,
    on_query_change: impl Fn(String) -> Message + 'a,
    on_select: impl Fn(usize) -> Message + 'a,
    on_cancel: impl Fn() -> Message + Clone + 'a,
) -> Element<'a, Message> {
    command_palette_styled(
        query,
        commands,
        selected_index,
        on_query_change,
        on_select,
        on_cancel,
        PaletteConfig::default(),
    )
}

/// Renders a command palette overlay with search input and custom configuration.
pub fn command_palette_styled<'a, Message: Clone + 'a>(
    query: &str,
    commands: &[Command<Message>],
    selected_index: usize,
    on_query_change: impl Fn(String) -> Message + 'a,
    on_select: impl Fn(usize) -> Message + 'a,
    on_cancel: impl Fn() -> Message + Clone + 'a,
    config: PaletteConfig,
) -> Element<'a, Message> {
    let on_cancel_clone = on_cancel.clone();
    let bg_opacity = config.background_opacity;

    // Filter commands based on query
    let filtered = filter_commands(query, commands);

    // Build command items - slim, no rounded corners
    let command_items: Vec<Element<'a, Message>> = filtered
        .iter()
        .enumerate()
        .map(|(display_index, (original_index, match_result))| {
            let cmd = &commands[*original_index];
            let is_selected = display_index == selected_index;
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
                        text::Style {
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

            // Build full row with optional shortcut on right (right-aligned)
            let content: Element<'a, Message> = if let Some(shortcut) = shortcut_display {
                Row::new()
                    .push(
                        container(left_content)
                            .width(Length::Fill)
                    )
                    .push(text(shortcut).size(11).style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
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
                Row::new()
                    .push(left_content)
                    .width(Length::Fill)
                    .into()
            };

            let on_select_msg = on_select(display_index);

            button(content)
                .on_press(on_select_msg)
                .padding([6, 10])
                .width(Length::Fill)
                .style(move |theme: &Theme, status| {
                    item_button_style(theme, is_selected, status)
                })
                .into()
        })
        .collect();

    let command_list = Column::with_children(command_items).spacing(1);

    // Search input with ID for focus management
    let search_input = text_input(&config.placeholder, query)
        .id(INPUT_ID)
        .on_input(on_query_change)
        .padding([8, 12])
        .size(14)
        .width(Length::Fill)
        .style(|theme: &Theme, _status| {
            let palette = theme.extended_palette();
            text_input::Style {
                background: iced::Background::Color(palette.background.base.color),
                border: iced::Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 0.0.into(),
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
        });

    // Close button
    let close_button = button(text("x").size(12))
        .on_press(on_cancel_clone())
        .padding([2, 6])
        .style(|_theme: &Theme, _status| button::Style::default());

    // Header with search input
    let header = row![search_input, close_button]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .padding([0, 8]);

    // Main palette content - sharp edges, minimal padding, with scrolling
    let palette_content = container(
        column![
            header,
            scrollable(
                container(command_list)
                    .padding([4, 0])
                    .width(Length::Fill)
            )
            .height(config.max_height),
        ]
        .spacing(6)
        .padding([8, 0])
        .width(config.width),
    )
    .style(|theme: &Theme| palette_container_style(theme));

    // Full-screen overlay
    mouse_area(
        container(palette_content)
            .center(Length::Fill)
            .style(move |theme: &Theme| overlay_background_style(theme, bg_opacity)),
    )
    .on_press(on_cancel())
    .into()
}

/// Returns the filtered command indices for use with keyboard navigation.
/// Call this to get the original command index when the user confirms selection.
pub fn get_filtered_command_index<Message>(
    query: &str,
    commands: &[Command<Message>],
    selected_display_index: usize,
) -> Option<usize> {
    let filtered = filter_commands(query, commands);
    filtered.get(selected_display_index).map(|(idx, _)| *idx)
}

/// Returns the count of filtered commands for bounds checking.
pub fn get_filtered_count<Message>(query: &str, commands: &[Command<Message>]) -> usize {
    filter_commands(query, commands).len()
}

fn item_button_style(theme: &Theme, is_selected: bool, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let (background, text_color) = if is_selected {
        // Selected state - primary color
        (
            Some(iced::Background::Color(palette.primary.base.color)),
            palette.primary.base.text,
        )
    } else {
        match status {
            button::Status::Hovered | button::Status::Pressed => {
                // Hover state - subtle highlight
                (
                    Some(iced::Background::Color(palette.background.strong.color)),
                    palette.background.base.text,
                )
            }
            _ => {
                // Normal state - no background
                (None, palette.background.base.text)
            }
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

fn palette_container_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(iced::Background::Color(palette.background.weak.color)),
        border: iced::Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..container::Style::default()
    }
}

fn overlay_background_style(theme: &Theme, opacity: f32) -> container::Style {
    let palette = theme.extended_palette();
    let bg = palette.background.base.color;
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            bg.r, bg.g, bg.b, opacity,
        ))),
        ..container::Style::default()
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
        // Skip indices that are out of bounds
        if idx >= chars.len() {
            continue;
        }
        // Add non-highlighted segment before this match
        if idx > last_end {
            let segment: String = chars[last_end..idx].iter().collect();
            spans.push(Span::new(segment));
        }
        // Add highlighted character
        let ch: String = chars[idx..idx + 1].iter().collect();
        spans.push(Span::new(ch).color(highlight_color));
        last_end = idx + 1;
    }

    // Add remaining non-highlighted text
    if last_end < chars.len() {
        let segment: String = chars[last_end..].iter().collect();
        spans.push(Span::new(segment));
    }

    Rich::with_spans(spans).size(13).into()
}
