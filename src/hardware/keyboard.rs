use crossterm::event::{KeyModifiers, poll, read};
use std::io;
use std::time::Duration;

/// Providing Keyboard Input independent of an implementation.
pub trait KeyboardInputProvider {
    /// Checks if input is available, does not block.
    fn check_input_available(&mut self) -> io::Result<bool>;
    /// Provides input if `check_input_available` returned `true`, panics otherwise.
    fn get_input_character(&mut self) -> char;
    /// True if CTRL-C was triggered
    fn is_interrupted(&self) -> bool;
}

pub struct TerminalInputProvider {
    is_char_available: bool,
    available_char: Option<char>,
    is_interrupted: bool,
}
impl TerminalInputProvider {
    pub const fn new() -> Self {
        Self {
            is_char_available: false,
            available_char: None,
            is_interrupted: false,
        }
    }
}
impl KeyboardInputProvider for TerminalInputProvider {
    fn check_input_available(&mut self) -> io::Result<bool> {
        if poll(Duration::from_secs(0))?
            && let Some(event) = read()?.as_key_press_event()
            && let Some(c) = event.code.as_char()
        {
            if c == 'c' && event.modifiers == KeyModifiers::CONTROL {
                self.is_interrupted = true;
            } else {
                self.is_char_available = true;
                self.available_char = Some(c);
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn get_input_character(&mut self) -> char {
        self.available_char
            .unwrap_or_else(|| panic!("No input available"))
    }
    fn is_interrupted(&self) -> bool {
        self.is_interrupted
    }
}
