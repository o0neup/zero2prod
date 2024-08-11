use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_whitespace_or_empty = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', ' ', '[', ']'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

        if is_whitespace_or_empty || contains_forbidden_characters || is_too_long {
            Err(format!("{} is not a valid subscriber name!", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn valid_name_is_parsed_successfully() {
        let name = "artyomka".to_string();
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let s = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(s));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_invalid() {
        let s = "n".repeat(257);
        assert_err!(SubscriberName::parse(s));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let s = "       ".to_string();
        assert_err!(SubscriberName::parse(s));
    }

    #[test]
    fn empty_string_is_rejected() {
        let s = "".to_string();
        assert_err!(SubscriberName::parse(s));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}', ' ', '[', ']'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }
}
