use crossterm::event::{KeyEvent, KeyCode};

pub struct InputTextWidget {
    content: String,
    cursor: Option<usize>,
}

impl InputTextWidget {
    pub fn new(content: Option<String>) -> InputTextWidget {
        InputTextWidget {
            content: content.unwrap_or_default(),
            cursor: None,
        }
    }

    pub fn key_pressed(&mut self, key_event: KeyEvent) {
        if let Some(mut cursor) = self.cursor {
            let KeyEvent{code, modifiers} = key_event;
            match code {
                KeyCode::Char(character) => {
                    self.content.insert(cursor, character);
                    cursor += 1;
                }
                KeyCode::Delete => {
                    if cursor < self.content.len() {
                        self.content.remove(cursor);
                    }
                }
                KeyCode::Backspace => {
                    if cursor > 0 {
                        cursor -= 1;
                        self.content.remove(cursor);
                    }
                }
                KeyCode::Left => {
                    if cursor > 0 {
                        cursor -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor < self.content.len() {
                        cursor += 1;
                    }
                }
                KeyCode::Home => {
                    cursor = 0;
                }
                KeyCode::End => {
                    cursor = self.content.len();
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

pub struct InputCharWidget {
    content: char,
    cursor: bool,
}

impl InputCharWidget {
    pub fn new(content: char) -> InputCharWidget {
        InputCharWidget {
            content,
            cursor: false,
        }
    }

    pub fn key_pressed(&mut self, key_event: KeyEvent) {

    }

    pub fn focus(&mut self, value: bool) {
        self.cursor = value;
    }

    pub fn has_focus(&self) -> bool {
        self.cursor
    }

    pub fn content(&self) -> char {
        self.content
    }
}
