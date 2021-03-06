use std::cmp::Ordering;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// This crate is the wrapper of crossterm::event::KeyEvent.
/// Implementing Ord trait is necessary for being keys of BTreeMap.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct KeyEventWrapper {
    inner: KeyEvent,
}

impl KeyEventWrapper {
    pub fn new(key_event: KeyEvent) -> Self {
        KeyEventWrapper { inner: key_event }
    }

    pub fn code(&self) -> KeyCode {
        self.inner.code
    }

    pub fn modifier(&self) -> KeyModifiers {
        self.inner.modifiers
    }
}

impl Ord for KeyEventWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut self_is_longer = true;
        let self_str = self.to_string();
        let other_str = other.to_string();
        let (b1, b2) = if self_str.len() < other_str.len() {
            (self_str.as_bytes(), other_str.as_bytes())
        } else {
            self_is_longer = false;
            (other_str.as_bytes(), self_str.as_bytes())
        };
        for (i, c) in b1.iter().enumerate() {
            match c.cmp(&b2[i]) {
                Ordering::Less => {
                    if self_is_longer {
                        return Ordering::Less;
                    } else {
                        return Ordering::Greater;
                    }
                }
                Ordering::Greater => {
                    if self_is_longer {
                        return Ordering::Greater;
                    } else {
                        return Ordering::Less;
                    }
                }
                _ => {}
            }
        }
        if self_str.len() == other_str.len() {
            Ordering::Equal
        } else if self_is_longer {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for KeyEventWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ToString for KeyEventWrapper {
    fn to_string(&self) -> String {
        let code = match self.inner.code {
            KeyCode::Char(c) => {
                if c == ' ' {
                    Some("SPC".to_string())
                } else if c == '*' {
                    Some("Chars".to_string())
                } else {
                    Some(c.to_uppercase().to_string())
                }
            }
            KeyCode::Backspace => Some("BackSpace".to_string()),
            KeyCode::Left => Some("←".to_string()),
            KeyCode::Up => Some("↑".to_string()),
            KeyCode::Right => Some("→".to_string()),
            KeyCode::Down => Some("↓".to_string()),
            KeyCode::Tab => Some("TAB".to_string()),
            KeyCode::Enter => Some("ENTER".to_string()),
            KeyCode::Esc => Some("Esc".to_string()),
            _ => None,
        };
        let modifier = match self.inner.modifiers {
            KeyModifiers::CONTROL => Some("Ctrl".to_string()),
            _ => None,
        };
        format!(
            "{}{}{}",
            code.unwrap_or_default(),
            if modifier.is_some() { "+" } else { "" },
            modifier.unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getter_fn() {
        let input = KeyEventWrapper::new(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(KeyCode::Enter, input.code());
        assert_eq!(KeyModifiers::NONE, input.modifier());
    }

    fn test_case_to_string(key_event: KeyEvent, expected: &str) {
        assert_eq!(
            expected.to_string(),
            KeyEventWrapper::new(key_event).to_string(),
        );
    }

    #[test]
    fn test_to_string() {
        test_case_to_string(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            "C+Ctrl",
        );
        test_case_to_string(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE), "SPC");
        test_case_to_string(
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            "BackSpace",
        );
        test_case_to_string(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), "Esc");
        test_case_to_string(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE), "");
    }

    #[test]
    fn test_cmp() {
        // A+Ctrl < B+Ctrl
        let one = KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
        let other = KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        assert!(one < other);
        assert!(other > one);
        // Z+Ctrl == Z+Ctrl
        let one = KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        let other = KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert!(one == other);
        assert!(other == one);
        // G+Ctrl < G
        let one = KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL));
        let other = KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        assert!(one > other);
        assert!(other < one);
    }
}
