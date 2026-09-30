//! Keyboard key classification. Unknown named keys are never editing commands.

/// Map explicit text controls or one printable Unicode scalar to tool input.
/// This intentionally does not interpret grapheme clusters or IME composition.
pub fn key_character(key: &str) -> Option<char> {
    match key {
        "Enter" => Some('\n'),
        "Backspace" => Some('\x08'),
        "Delete" => Some('\0'),
        "Tab" => Some('\t'),
        _ => {
            let mut chars = key.chars();
            let ch = chars.next()?;
            (chars.next().is_none() && !ch.is_control()).then_some(ch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_unicode_scalars_and_space_not_byte_length() {
        for ch in ['A', ' ', 'é', '界', '😀'] {
            assert_eq!(key_character(&ch.to_string()), Some(ch));
        }
    }

    #[test]
    fn only_explicit_named_controls_produce_control_input() {
        for key in [
            "ArrowRight",
            "Dead",
            "Shift",
            "F1",
            "",
            "\0",
            "\u{7f}",
            "e\u{301}",
        ] {
            assert_eq!(key_character(key), None, "{key:?}");
        }
        assert_eq!(key_character("Delete"), Some('\0'));
        assert_eq!(key_character("Backspace"), Some('\x08'));
        assert_eq!(key_character("Enter"), Some('\n'));
    }
}
