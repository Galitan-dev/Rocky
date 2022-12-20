use colored::Colorize;
use rustyline::hint::{Hint, Hinter};
use rustyline::Context;
use rustyline_derive::{Completer, Helper, Highlighter, Validator};
use std::collections::HashSet;

#[derive(Completer, Helper, Validator, Highlighter, Clone)]
pub struct RkHinter {
    pub hints: HashSet<CommandHint>,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone)]
pub struct CommandHint {
    display: String,
    display_color: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display_color
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> CommandHint {
        assert!(text.starts_with(complete_up_to));
        CommandHint {
            display: text.into(),
            display_color: text.bright_black().to_string(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> CommandHint {
        CommandHint {
            display: self.display[strip_chars..].to_owned(),
            display_color: self.display[strip_chars..].bright_black().to_string(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for RkHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() || line.len() < 2 {
            return None;
        }

        self.hints
            .iter()
            .filter_map(|hint| {
                if hint.display.starts_with(line) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}

pub fn rk_hints() -> HashSet<CommandHint> {
    let mut set = HashSet::new();
    set.insert(CommandHint::new("!quit", "!quit"));
    set.insert(CommandHint::new("!program", "!program"));
    set.insert(CommandHint::new("!clear_program", "!clear_program"));
    set.insert(CommandHint::new("!clear_registers", "!clear_registers"));
    set.insert(CommandHint::new("!registers", "!registers"));
    set.insert(CommandHint::new("!symbols", "!symbols"));
    set.insert(CommandHint::new(
        "!load_file path/to/file.rk",
        "!load_file ",
    ));
    set.insert(CommandHint::new("!spawn path/to/file.rk", "!spawn "));
    set
}
