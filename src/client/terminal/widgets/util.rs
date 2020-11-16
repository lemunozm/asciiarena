use tui::layout::{Rect};

use crossterm::event::{KeyEvent, KeyCode};

pub fn centered_area(base: Rect, dimension: (u16, u16)) -> Rect {
    let width_diff = base.width as i16 - dimension.0 as i16;
    let height_diff = base.height as i16 - dimension.1 as i16;
    let x = if width_diff > 0 { base.x + width_diff as u16 / 2 } else { 0 };
    let y = if height_diff > 0 { base.y + height_diff as u16 / 2 } else { 0 };
    let width = if base.width > dimension.0 { dimension.0 } else { base.width };
    let height = if base.height > dimension.1 { dimension.1 } else { base.height };
    Rect::new(x, y, width, height)
}

pub fn vertically_centered(base: Rect, height: u16) -> Rect {
    let height_diff = base.height as i16 - height as i16;
    let y = if height_diff > 0 { base.y + height_diff as u16 / 2 } else { 0 };
    let height = if base.height > height { height } else { base.height };
    Rect::new(base.x, y, base.width, height)
}

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
