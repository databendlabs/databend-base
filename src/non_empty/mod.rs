//! Non-empty string types that guarantee the contained string is never empty.
//!
//! - [`NonEmptyStr`]: A borrowed non-empty string slice.
//! - [`NonEmptyString`]: An owned non-empty string.

use std::fmt;

/// A borrowed string slice guaranteed to be non-empty.
#[derive(Clone, Debug, Copy)]
pub struct NonEmptyStr<'a> {
    inner: &'a str,
}

impl<'a> NonEmptyStr<'a> {
    pub fn new(s: &'a str) -> Result<Self, &'static str> {
        if s.is_empty() {
            return Err("input str is empty");
        }
        Ok(NonEmptyStr { inner: s })
    }

    pub fn as_str(&self) -> &str {
        self.inner
    }
}

impl fmt::Display for NonEmptyStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl AsRef<str> for NonEmptyStr<'_> {
    fn as_ref(&self) -> &str {
        self.inner
    }
}

/// An owned string guaranteed to be non-empty.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NonEmptyString {
    inner: String,
}

impl NonEmptyString {
    pub fn new(s: impl ToString) -> Result<Self, &'static str> {
        let s = s.to_string();
        if s.is_empty() {
            return Err("input is empty");
        }
        Ok(NonEmptyString { inner: s })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn as_non_empty_str(&self) -> NonEmptyStr<'_> {
        NonEmptyStr { inner: &self.inner }
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl PartialEq<str> for NonEmptyString {
    fn eq(&self, other: &str) -> bool {
        self.inner == *other
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<'a> From<NonEmptyStr<'a>> for NonEmptyString {
    fn from(value: NonEmptyStr<'a>) -> Self {
        NonEmptyString {
            inner: value.inner.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_non_empty_str() {
        assert!(NonEmptyStr::new("").is_err());

        let s = NonEmptyStr::new("hello").unwrap();
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.to_string(), "hello");
        assert_eq!(s.as_ref(), "hello");
        assert_eq!(format!("{:?}", s), "NonEmptyStr { inner: \"hello\" }");

        // Copy
        let copied = s;
        let copied2 = s;
        assert_eq!(copied.as_str(), "hello");
        assert_eq!(copied2.as_str(), "hello");
    }

    #[test]
    fn test_non_empty_string() {
        assert!(NonEmptyString::new("").is_err());

        let s = NonEmptyString::new("hello").unwrap();
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.to_string(), "hello");
        assert_eq!(s.as_ref(), "hello");
        assert_eq!(s, *"hello");
        assert_eq!(format!("{:?}", s), "NonEmptyString { inner: \"hello\" }");

        let borrowed = s.as_non_empty_str();
        assert_eq!(borrowed.as_str(), "hello");

        // Clone
        let cloned = s.clone();
        assert_eq!(cloned.as_str(), "hello");
    }

    #[test]
    fn test_non_empty_string_hash() {
        let mut set = HashSet::new();
        set.insert(NonEmptyString::new("a").unwrap());
        set.insert(NonEmptyString::new("b").unwrap());
        set.insert(NonEmptyString::new("a").unwrap());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_conversion() {
        let borrowed = NonEmptyStr::new("hello").unwrap();
        let owned: NonEmptyString = borrowed.into();
        assert_eq!(owned.as_str(), "hello");
    }

    #[test]
    fn test_whitespace_and_unicode() {
        // Whitespace is valid (non-empty)
        assert!(NonEmptyStr::new(" ").is_ok());
        assert!(NonEmptyStr::new("\t").is_ok());
        assert!(NonEmptyStr::new("\n").is_ok());
        assert!(NonEmptyString::new(" ").is_ok());

        // Unicode
        let s = NonEmptyString::new("你好").unwrap();
        assert_eq!(s.as_str(), "你好");

        let s = NonEmptyStr::new("🦀").unwrap();
        assert_eq!(s.as_str(), "🦀");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let s = NonEmptyString::new("hello").unwrap();
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#"{"inner":"hello"}"#);

        let deserialized: NonEmptyString = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, s);
    }
}
