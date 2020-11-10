pub fn is_valid_character_name(name: &str) -> bool {
    name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase())
}

pub fn is_valid_character(character: char) -> bool {
    character.is_ascii_uppercase()
}

pub mod format {
    use std::borrow::{Borrow};

    pub fn character_list(list: impl IntoIterator<Item = impl Borrow<char>>) -> String {
        let mut formatted = String::new();
        let mut it = list.into_iter();
        if let Some(character) = it.next() {
            formatted.push(*character.borrow());
            for character in it {
                formatted.push_str(&format!(", {}", *character.borrow()));
            }
        }
        formatted
    }

    pub fn character_points_list(list: impl IntoIterator<Item = (char, usize)>) -> String {
        let mut formatted = String::new();
        let mut it = list.into_iter();
        if let Some((character, points)) = it.next() {
            formatted.push_str(&format!("{}: {}", character, points));
            for (character, points) in it {
                formatted.push_str(&format!(", {}: {}", character, points));
            }
        }
        formatted
    }
}


