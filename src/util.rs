pub fn is_valid_character_name(name: &str) -> bool {
    name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase())
}

pub fn is_valid_character(character: char) -> bool {
    character.is_ascii_uppercase()
}

pub mod format {
    pub fn items_to_string<I>(items: I) -> String
    where I: IntoIterator,
          I::Item: std::fmt::Display,
    {
        let mut formatted = String::new();
        let mut it = items.into_iter();
        if let Some(item) = it.next() {
            formatted.push_str(&format!("{}", item));
            for item in it {
                formatted.push_str(&format!(", {}", item));
            }
        }
        formatted
    }

    pub fn pair_items_to_string<I, D1, D2>(items: I) -> String
    where D1: std::fmt::Display,
          D2: std::fmt::Display,
          I: IntoIterator<Item = (D1, D2)>,
    {
        let mut formatted = String::new();
        let mut it = items.into_iter();
        if let Some((id, content)) = it.next() {
            formatted.push_str(&format!("{}: {}", id, content));
            for (id, content) in it {
                formatted.push_str(&format!(", {}: {}", id, content));
            }
        }
        formatted
    }
}


