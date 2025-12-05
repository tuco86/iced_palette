//! Subscription helpers for command palette keyboard handling.
//!
//! Note: Due to Iced's design, subscription closures cannot capture variables.
//! The helpers here provide utility functions for working with keyboard events
//! but the actual subscription logic must be implemented in the application.

use crate::{Command, CommandAction, Shortcut};
use iced::keyboard;

/// Checks if a keyboard event matches the palette toggle shortcut (Ctrl+Space).
pub fn is_toggle_shortcut(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
    modifiers.command() && *key == keyboard::Key::Named(keyboard::key::Named::Space)
}

/// Finds if a keyboard event matches any command shortcut.
/// Returns the command ID if found.
pub fn find_matching_shortcut<'a, Message>(
    commands: &'a [Command<Message>],
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<&'static str> {
    for cmd in commands {
        if let Some(ref shortcut) = cmd.shortcut {
            if shortcut.matches(key, modifiers) {
                return Some(cmd.id);
            }
        }
        // Check submenus recursively
        if let CommandAction::Submenu(ref subcmds) = cmd.action {
            if let Some(id) = find_matching_shortcut(subcmds, key, modifiers) {
                return Some(id);
            }
        }
    }
    None
}

/// Calculates the next index when navigating up in a list with wrapping.
pub fn navigate_up(current_index: usize, item_count: usize) -> usize {
    if current_index == 0 {
        if item_count > 0 { item_count - 1 } else { 0 }
    } else {
        current_index - 1
    }
}

/// Calculates the next index when navigating down in a list with wrapping.
pub fn navigate_down(current_index: usize, item_count: usize) -> usize {
    if item_count == 0 {
        0
    } else if current_index >= item_count - 1 {
        0
    } else {
        current_index + 1
    }
}

/// Collects all shortcuts from commands, including those in submenus.
pub fn collect_shortcuts<Message>(commands: &[Command<Message>]) -> Vec<(&'static str, Shortcut)> {
    let mut result = Vec::new();
    for cmd in commands {
        if let Some(ref shortcut) = cmd.shortcut {
            result.push((cmd.id, shortcut.clone()));
        }
        // Recurse into submenus
        if let CommandAction::Submenu(ref subcmds) = cmd.action {
            result.extend(collect_shortcuts(subcmds));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMessage {
        Action1,
        Action2,
        Sub1,
    }

    #[test]
    fn test_collect_shortcuts() {
        let commands = vec![
            command("cmd1", "Command 1")
                .shortcut(Shortcut::cmd('n'))
                .action(TestMessage::Action1),
            command("cmd2", "Command 2")
                .shortcut(Shortcut::cmd('t'))
                .action(TestMessage::Action2),
            command("submenu", "Submenu").submenu(vec![
                command("sub1", "Sub Command")
                    .shortcut(Shortcut::cmd('s'))
                    .action(TestMessage::Sub1),
            ]),
        ];

        let shortcuts = collect_shortcuts(&commands);
        assert_eq!(shortcuts.len(), 3);
        assert_eq!(shortcuts[0].0, "cmd1");
        assert_eq!(shortcuts[1].0, "cmd2");
        assert_eq!(shortcuts[2].0, "sub1");
    }

    #[test]
    fn test_navigate_up_wrapping() {
        assert_eq!(navigate_up(0, 5), 4); // Wrap to end
        assert_eq!(navigate_up(3, 5), 2); // Normal
        assert_eq!(navigate_up(0, 0), 0); // Empty list
    }

    #[test]
    fn test_navigate_down_wrapping() {
        assert_eq!(navigate_down(4, 5), 0); // Wrap to start
        assert_eq!(navigate_down(2, 5), 3); // Normal
        assert_eq!(navigate_down(0, 0), 0); // Empty list
    }
}
