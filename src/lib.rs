//! # iced_palette
//!
//! A command palette library for Iced applications.
//!
//! ## Quick Start (Widget API)
//!
//! ```rust,ignore
//! use iced_palette::{Palette, PaletteState, Command, command};
//!
//! // In your application state:
//! struct App {
//!     palette: PaletteState,
//! }
//!
//! // Define commands:
//! let commands = vec![
//!     command("save", "Save File")
//!         .description("Save the current file")
//!         .action(Message::Save),
//! ];
//!
//! // In your view:
//! if self.palette.is_open() {
//!     stack![
//!         main_content,
//!         Palette::new(&self.palette, &commands)
//!             .on_query_change(Message::QueryChanged)
//!             .on_select(|id| Message::CommandSelected(id))
//!             .on_close(|| Message::PaletteClosed)
//!     ]
//! }
//! ```
//!
//! ## Helper Functions API (simpler, for external state management)
//!
//! ```rust,ignore
//! use iced_palette::{command_palette, Command, command};
//!
//! // In your view, use the command_palette helper
//! if palette_is_open {
//!     stack!(main_view, command_palette(...))
//! }
//! ```

mod command;
mod helpers;
mod palette;
mod search;
mod subscription;

// Widget API (recommended)
pub use palette::{Palette, PaletteState, PaletteStyle, focus_input as palette_focus};

// Command types
pub use command::{Category, Command, CommandAction, CommandBuilder, Shortcut, command, find_by_shortcut};

// Helper functions API (for simpler use cases)
pub use helpers::{command_palette, command_palette_styled, get_filtered_command_index, get_filtered_count, focus_input, INPUT_ID, PaletteConfig};

// Search utilities
pub use search::{fuzzy_match, filter_commands, FuzzyMatch};

// Subscription helpers
pub use subscription::{is_toggle_shortcut, find_matching_shortcut, navigate_up, navigate_down, collect_shortcuts};
