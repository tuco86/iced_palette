//! Command types for the palette.

use iced::keyboard;
use std::sync::Arc;

/// Unique identifier for a command.
pub type CommandId = &'static str;

/// Category for grouping commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Category {
    /// Unique identifier for the category.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Sort order (lower = first).
    pub order: u32,
}

impl Category {
    /// Creates a new category.
    pub const fn new(id: &'static str, name: &'static str, order: u32) -> Self {
        Self { id, name, order }
    }

    /// File operations category.
    pub const FILE: Category = Category::new("file", "File", 100);
    /// Edit operations category.
    pub const EDIT: Category = Category::new("edit", "Edit", 200);
    /// View operations category.
    pub const VIEW: Category = Category::new("view", "View", 300);
    /// Navigation operations category.
    pub const GOTO: Category = Category::new("goto", "Go to", 400);
    /// Help and documentation category.
    pub const HELP: Category = Category::new("help", "Help", 900);
}

/// Keyboard shortcut for a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    /// The key to press.
    pub key: keyboard::Key,
    /// Required modifiers.
    pub modifiers: keyboard::Modifiers,
}

impl Shortcut {
    /// Creates a new shortcut with the given key and modifiers.
    pub fn new(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Creates Cmd/Ctrl + key shortcut.
    pub fn cmd(c: char) -> Self {
        Self {
            key: keyboard::Key::Character(c.to_string().into()),
            modifiers: keyboard::Modifiers::COMMAND,
        }
    }

    /// Creates Cmd/Ctrl + Shift + key shortcut.
    pub fn cmd_shift(c: char) -> Self {
        Self {
            key: keyboard::Key::Character(c.to_string().into()),
            modifiers: keyboard::Modifiers::COMMAND.union(keyboard::Modifiers::SHIFT),
        }
    }

    /// Creates Ctrl + key shortcut (explicit, not platform-aware).
    pub fn ctrl(c: char) -> Self {
        Self {
            key: keyboard::Key::Character(c.to_string().into()),
            modifiers: keyboard::Modifiers::CTRL,
        }
    }

    /// Creates Alt + key shortcut.
    pub fn alt(c: char) -> Self {
        Self {
            key: keyboard::Key::Character(c.to_string().into()),
            modifiers: keyboard::Modifiers::ALT,
        }
    }

    /// Checks if this shortcut matches the given key press.
    pub fn matches(&self, key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        // Normalize key comparison (case-insensitive for characters)
        let key_matches = match (&self.key, key) {
            (
                keyboard::Key::Character(a),
                keyboard::Key::Character(b),
            ) => a.to_lowercase() == b.to_lowercase(),
            (a, b) => a == b,
        };

        key_matches && self.modifiers == modifiers
    }

    /// Returns display string for the shortcut.
    ///
    /// Uses platform-appropriate modifier symbols:
    /// - macOS: Cmd, Opt, Shift, Ctrl
    /// - Other: Ctrl, Alt, Shift
    pub fn display(&self) -> String {
        let mut parts = Vec::new();

        #[cfg(target_os = "macos")]
        {
            if self.modifiers.control() {
                parts.push("Ctrl");
            }
            if self.modifiers.alt() {
                parts.push("Opt");
            }
            if self.modifiers.shift() {
                parts.push("Shift");
            }
            if self.modifiers.command() {
                parts.push("Cmd");
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            if self.modifiers.control() || self.modifiers.command() {
                parts.push("Ctrl");
            }
            if self.modifiers.alt() {
                parts.push("Alt");
            }
            if self.modifiers.shift() {
                parts.push("Shift");
            }
        }

        // Add key
        let key_str = match &self.key {
            keyboard::Key::Character(c) => c.to_uppercase(),
            keyboard::Key::Named(named) => format!("{:?}", named),
            _ => "?".to_string(),
        };
        parts.push(&key_str);

        parts.join("+")
    }
}

/// A command that can be executed from the palette.
#[derive(Clone)]
pub struct Command<Message> {
    /// Unique identifier.
    pub id: CommandId,

    /// Display name shown in the palette.
    pub name: String,

    /// Optional description/help text.
    pub description: Option<String>,

    /// Category for grouping (e.g., "file", "edit", "view").
    pub category: Option<&'static str>,

    /// Keyboard shortcut for direct activation.
    pub shortcut: Option<Shortcut>,

    /// Keywords for improved search (not displayed).
    pub keywords: Vec<String>,

    /// Whether command is currently enabled.
    pub enabled: bool,

    /// Action to perform when executed.
    pub action: CommandAction<Message>,
}

/// How a command produces its message.
#[derive(Clone)]
pub enum CommandAction<Message> {
    /// Produce a specific message.
    Message(Message),

    /// Call a function to get the message (for dynamic commands).
    Callback(Arc<dyn Fn() -> Message + Send + Sync>),

    /// Open a submenu/nested command list.
    Submenu(Vec<Command<Message>>),
}

impl<Message> Command<Message> {
    /// Creates a new command.
    pub fn new(id: CommandId, name: impl Into<String>, action: CommandAction<Message>) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            category: None,
            shortcut: None,
            keywords: Vec::new(),
            enabled: true,
            action,
        }
    }
}

/// Builder for ergonomic command creation.
pub struct CommandBuilder<Message> {
    id: CommandId,
    name: String,
    description: Option<String>,
    category: Option<&'static str>,
    shortcut: Option<Shortcut>,
    keywords: Vec<String>,
    enabled: bool,
    _phantom: std::marker::PhantomData<Message>,
}

impl<Message> CommandBuilder<Message> {
    /// Creates a new command builder.
    pub fn new(id: CommandId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            category: None,
            shortcut: None,
            keywords: Vec::new(),
            enabled: true,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the category.
    pub fn category(mut self, category: &'static str) -> Self {
        self.category = Some(category);
        self
    }

    /// Sets the keyboard shortcut.
    pub fn shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Adds a keyword for search.
    pub fn keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    /// Adds multiple keywords.
    pub fn keywords(mut self, keywords: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords.extend(keywords.into_iter().map(Into::into));
        self
    }

    /// Sets whether the command is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Builds the command with a message action.
    pub fn action(self, message: Message) -> Command<Message>
    where
        Message: Clone,
    {
        Command {
            id: self.id,
            name: self.name,
            description: self.description,
            category: self.category,
            shortcut: self.shortcut,
            keywords: self.keywords,
            enabled: self.enabled,
            action: CommandAction::Message(message),
        }
    }

    /// Builds the command with a submenu.
    pub fn submenu(self, commands: Vec<Command<Message>>) -> Command<Message> {
        Command {
            id: self.id,
            name: self.name,
            description: self.description,
            category: self.category,
            shortcut: self.shortcut,
            keywords: self.keywords,
            enabled: self.enabled,
            action: CommandAction::Submenu(commands),
        }
    }
}

/// Creates a new command builder.
///
/// # Example
///
/// ```rust
/// use iced_palette::command;
///
/// let cmd = command("save", "Save File")
///     .description("Save the current file to disk")
///     .category("file")
///     .action(Message::Save);
/// ```
pub fn command<Message>(id: CommandId, name: impl Into<String>) -> CommandBuilder<Message> {
    CommandBuilder::new(id, name)
}

/// Finds a command that matches the given keyboard shortcut.
///
/// Returns the index and a reference to the matching command if found.
pub fn find_by_shortcut<'a, Message>(
    commands: &'a [Command<Message>],
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<(usize, &'a Command<Message>)> {
    commands.iter().enumerate().find(|(_, cmd)| {
        cmd.shortcut
            .as_ref()
            .map(|s| s.matches(key, modifiers))
            .unwrap_or(false)
    })
}
