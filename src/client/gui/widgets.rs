use crossterm::event::{KeyEvent, KeyCode};

pub struct InputText {
    content: String,
    cursor: Option<usize>,
}

impl InputText {
    pub fn new(content: Option<String>) -> InputText {
        InputText {
            content: content.unwrap_or_default(),
            cursor: None,
        }
    }

    pub fn key_pressed(&mut self, key_event: KeyEvent) {
        if let Some(ref mut cursor) = self.cursor {
            match key_event.code {
                KeyCode::Char(character) => {
                    self.content.insert(*cursor, character);
                    *cursor += 1;
                }
                KeyCode::Delete => {
                    if *cursor < self.content.len() {
                        self.content.remove(*cursor);
                    }
                }
                KeyCode::Backspace => {
                    if *cursor > 0 {
                        *cursor -= 1;
                        self.content.remove(*cursor);
                    }
                }
                KeyCode::Left => {
                    if *cursor > 0 {
                        *cursor -= 1;
                    }
                }
                KeyCode::Right => {
                    if *cursor < self.content.len() {
                        *cursor += 1;
                    }
                }
                KeyCode::Home => {
                    *cursor = 0;
                }
                KeyCode::End => {
                    *cursor = self.content.len();
                }
                _ => (),
            }
        }
    }

    pub fn focus(&mut self, value: bool) {
        if value {
            if self.cursor.is_none() {
                self.cursor = Some(0);
            }
        }
        else {
            self.cursor = None;
        }
    }

    pub fn has_focus(&self) -> bool {
        self.cursor.is_some()
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    pub fn cursor_position(&self) -> Option<usize> {
        self.cursor
    }
}

pub struct InputCapitalLetter {
    content: Option<char>,
    focus: bool,
}

impl InputCapitalLetter {
    pub fn new(content: Option<char>) -> InputCapitalLetter {
        InputCapitalLetter {
            content,
            focus: false,
        }
    }

    pub fn key_pressed(&mut self, key_event: KeyEvent) {
        if self.focus {
            match key_event.code {
                KeyCode::Char(character) => {
                    if character.is_ascii_alphabetic() {
                        self.content = Some(character.to_ascii_uppercase());
                    }
                }
                KeyCode::Delete => {
                    self.content = None;
                }
                KeyCode::Backspace => {
                    self.content = None;
                }
                _ => (),
            }
        }
    }

    pub fn focus(&mut self, value: bool) {
        self.focus = value;
    }

    pub fn has_focus(&self) -> bool {
        self.focus
    }

    pub fn content(&self) -> Option<char> {
        self.content
    }
}
