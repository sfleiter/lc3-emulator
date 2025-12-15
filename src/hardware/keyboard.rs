use crossterm::event::{poll, read};
use std::io;
use std::time::Duration;

pub trait KeyboardInputProvider {
    fn check_input_available(&mut self) -> io::Result<bool>;
    fn get_input_character(&mut self) -> char;
}

pub struct TerminalInputProvider {
    is_char_available: bool,
    available_char: Option<char>,
}
impl TerminalInputProvider {
    pub const fn new() -> Self {
        Self {
            is_char_available: false,
            available_char: None,
        }
    }
}
impl KeyboardInputProvider for TerminalInputProvider {
    fn check_input_available(&mut self) -> io::Result<bool> {
        if poll(Duration::from_secs(0))?
            && let Some(event) = read()?.as_key_event()
            && let Some(c) = event.code.as_char()
        {
            self.is_char_available = true;
            self.available_char = Some(c);
            return Ok(true);
        }
        Ok(false)
    }
    fn get_input_character(&mut self) -> char {
        self.available_char
            .unwrap_or_else(|| panic!("No input available"))
    }
}
