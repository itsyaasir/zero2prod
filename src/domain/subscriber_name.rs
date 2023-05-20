use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        // `.trim()` returns a val with trailing white-space like characters
        //     `.is_empty()` checks if the views contains any characters

        let is_empty_or_whitespace = s.trim().is_empty();

        //     a grapheme is defined by the Unicode standard as `perceived` character
        let is_too_long = s.graphemes(true).count() > 256;
        // one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        // Return `false` if any of our conditions have been violated
        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", s))
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
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_graphene_long_name_is_valid() {
        let name = "ß".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_invalid() {
        let name = "ß".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_invalid() {
        let name = " ".repeat(256);
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn empty_names_are_invalid() {
        let name = "".repeat(256);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_with_forbidden_characters_are_invalid() {
        for name in ["/", "(", ")", "\"", "<", ">", "\\", "{", "}"].iter() {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_vaid_name_is_valid() {
        let name = "Yasir".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
