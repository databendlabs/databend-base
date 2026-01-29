//! Non-empty string types that guarantee the contained string is never empty.
//!
//! - [`NonEmptyStr`]: A borrowed non-empty string slice.
//! - [`NonEmptyString`]: An owned non-empty string.

use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

/// A borrowed string slice guaranteed to be non-empty.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.inner
    }
}

impl Deref for NonEmptyStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl Deref for NonEmptyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Borrow<str> for NonEmptyStr<'_> {
    fn borrow(&self) -> &str {
        self.inner
    }
}

impl Borrow<str> for NonEmptyString {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl PartialEq<str> for NonEmptyStr<'_> {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<String> for NonEmptyStr<'_> {
    fn eq(&self, other: &String) -> bool {
        self.inner == other.as_str()
    }
}

impl PartialEq<NonEmptyString> for NonEmptyStr<'_> {
    fn eq(&self, other: &NonEmptyString) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq<String> for NonEmptyString {
    fn eq(&self, other: &String) -> bool {
        self.inner == *other
    }
}

impl PartialEq<NonEmptyStr<'_>> for NonEmptyString {
    fn eq(&self, other: &NonEmptyStr<'_>) -> bool {
        self.inner == other.inner
    }
}

impl AsRef<[u8]> for NonEmptyStr<'_> {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl AsRef<[u8]> for NonEmptyString {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl FromStr for NonEmptyString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NonEmptyString::new(s)
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("input is empty");
        }
        Ok(NonEmptyString { inner: value })
    }
}

impl TryFrom<&str> for NonEmptyString {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        NonEmptyString::new(value)
    }
}

impl<'a> TryFrom<&'a str> for NonEmptyStr<'a> {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        NonEmptyStr::new(value)
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
        assert_eq!(AsRef::<str>::as_ref(&s), "hello");
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
        assert_eq!(AsRef::<str>::as_ref(&s), "hello");
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
    fn test_non_empty_str_hash_and_eq() {
        let mut set = HashSet::new();
        set.insert(NonEmptyStr::new("a").unwrap());
        set.insert(NonEmptyStr::new("b").unwrap());
        set.insert(NonEmptyStr::new("a").unwrap());
        assert_eq!(set.len(), 2);

        let s1 = NonEmptyStr::new("hello").unwrap();
        let s2 = NonEmptyStr::new("hello").unwrap();
        let s3 = NonEmptyStr::new("world").unwrap();
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_ordering() {
        let a = NonEmptyStr::new("a").unwrap();
        let b = NonEmptyStr::new("b").unwrap();
        assert!(a < b);

        let owned_a = NonEmptyString::new("a").unwrap();
        let owned_b = NonEmptyString::new("b").unwrap();
        assert!(owned_a < owned_b);

        // Sorting
        let mut strs = [
            NonEmptyStr::new("c").unwrap(),
            NonEmptyStr::new("a").unwrap(),
            NonEmptyStr::new("b").unwrap(),
        ];
        strs.sort();
        assert_eq!(strs[0].as_str(), "a");
        assert_eq!(strs[1].as_str(), "b");
        assert_eq!(strs[2].as_str(), "c");
    }

    #[test]
    fn test_deref() {
        let s = NonEmptyStr::new("hello").unwrap();
        // Deref allows using str methods directly
        assert!(s.starts_with("he"));
        assert_eq!(s.len(), 5);

        let owned = NonEmptyString::new("world").unwrap();
        assert!(owned.ends_with("ld"));
        assert_eq!(owned.len(), 5);
    }

    #[test]
    fn test_borrow() {
        use std::collections::HashMap;

        let mut map: HashMap<NonEmptyString, i32> = HashMap::new();
        map.insert(NonEmptyString::new("key").unwrap(), 42);

        // Borrow<str> allows lookup with &str
        assert_eq!(map.get("key"), Some(&42));
    }

    #[test]
    fn test_cross_type_equality() {
        let borrowed = NonEmptyStr::new("hello").unwrap();
        let owned = NonEmptyString::new("hello").unwrap();

        // NonEmptyStr vs NonEmptyString
        assert_eq!(borrowed, owned);
        assert_eq!(owned, borrowed);

        // vs str
        assert_eq!(borrowed, *"hello");
        assert_eq!(owned, *"hello");

        // vs String
        assert_eq!(borrowed, String::from("hello"));
        assert_eq!(owned, String::from("hello"));
    }

    #[test]
    fn test_as_ref_bytes() {
        let s = NonEmptyStr::new("abc").unwrap();
        let bytes: &[u8] = s.as_ref();
        assert_eq!(bytes, b"abc");

        let owned = NonEmptyString::new("xyz").unwrap();
        let bytes: &[u8] = owned.as_ref();
        assert_eq!(bytes, b"xyz");
    }

    #[test]
    fn test_from_str() {
        let parsed: NonEmptyString = "hello".parse().unwrap();
        assert_eq!(parsed.as_str(), "hello");

        let err: Result<NonEmptyString, _> = "".parse();
        assert!(err.is_err());
    }

    #[test]
    fn test_try_from() {
        // TryFrom<String>
        let owned: NonEmptyString = String::from("hello").try_into().unwrap();
        assert_eq!(owned.as_str(), "hello");

        let err: Result<NonEmptyString, _> = String::new().try_into();
        assert!(err.is_err());

        // TryFrom<&str> for NonEmptyString
        let owned: NonEmptyString = "world".try_into().unwrap();
        assert_eq!(owned.as_str(), "world");

        // TryFrom<&str> for NonEmptyStr
        let borrowed: NonEmptyStr = "test".try_into().unwrap();
        assert_eq!(borrowed.as_str(), "test");
    }

    #[test]
    fn test_into_string() {
        let owned = NonEmptyString::new("hello").unwrap();
        let s: String = owned.into();
        assert_eq!(s, "hello");
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
