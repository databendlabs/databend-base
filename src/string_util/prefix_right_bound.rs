/// Computes the exclusive right bound for a string prefix range query.
///
/// Given a prefix `p`, returns `Some(bound)` where `bound` is the smallest string
/// that is greater than all strings starting with `p`. This enables efficient
/// range queries like `prefix..right_bound` to match all strings with the prefix.
///
/// Returns `None` if no valid bound exists:
/// - Empty string (no prefix to bound)
/// - String ending with `char::MAX` repeatedly with no incrementable character
///
/// # Algorithm
///
/// Starting from the last character, find the first character that can be
/// incremented (is not `char::MAX`). Increment it and return the prefix up to
/// and including that character.
///
/// # Examples
///
/// ```
/// use databend_base::string_util::prefix_right_bound;
///
/// // Simple ASCII prefix
/// assert_eq!(prefix_right_bound("foo"), Some("fop".to_string()));
///
/// // Unicode prefix
/// assert_eq!(prefix_right_bound("日本"), Some("日札".to_string()));
///
/// // Empty string has no bound
/// assert_eq!(prefix_right_bound(""), None);
/// ```
pub fn prefix_right_bound(p: &str) -> Option<String> {
    if p.is_empty() {
        return None;
    }

    let chars: Vec<char> = p.chars().collect();

    // Find the rightmost character that can be incremented
    for i in (0..chars.len()).rev() {
        if let Some(next_char) = char::from_u32(chars[i] as u32 + 1) {
            // Build result: prefix up to i, then incremented char
            let mut result: String = chars[..i].iter().collect();
            result.push(next_char);
            return Some(result);
        }
        // chars[i] is char::MAX, continue to previous character
    }

    // All characters are char::MAX, no valid bound exists
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> Option<String> {
        Some(v.to_string())
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(prefix_right_bound(""), None);
    }

    #[test]
    fn test_single_char() {
        assert_eq!(prefix_right_bound("a"), s("b"));
        assert_eq!(prefix_right_bound("z"), s("{"));
        assert_eq!(prefix_right_bound("A"), s("B"));
    }

    #[test]
    fn test_ascii_strings() {
        assert_eq!(prefix_right_bound("foo"), s("fop"));
        assert_eq!(prefix_right_bound("bar"), s("bas"));
        assert_eq!(prefix_right_bound("hello"), s("hellp"));
        assert_eq!(prefix_right_bound("abc"), s("abd"));
        assert_eq!(prefix_right_bound("foo/"), s("foo0"));
        assert_eq!(prefix_right_bound("hello!"), s("hello\""));
    }

    #[test]
    fn test_unicode_strings() {
        // Japanese: 日本 -> 日札 (本 U+672C -> 札 U+672D)
        assert_eq!(prefix_right_bound("日本"), s("日札"));

        // Chinese: 中文 -> 中斈 (文 U+6587 -> 斈 U+6588)
        assert_eq!(prefix_right_bound("中文"), s("中斈"));

        // Emoji
        assert_eq!(prefix_right_bound("🎉"), s("🎊"));
        assert_eq!(prefix_right_bound("😀"), s("😁"));
        assert_eq!(prefix_right_bound("foo💯"), s("foo💰"));

        // Non-ASCII Latin
        assert_eq!(prefix_right_bound("ñ"), s("\u{00f2}"));
    }

    #[test]
    fn test_mixed_content() {
        assert_eq!(prefix_right_bound("foo/bar"), s("foo/bas"));
        assert_eq!(prefix_right_bound("path/to/file"), s("path/to/filf"));
    }

    #[test]
    fn test_trailing_max_chars() {
        // When last char is max, should increment previous char
        let single_trailing = format!("a{}", char::MAX);
        assert_eq!(prefix_right_bound(&single_trailing), s("b"));

        let with_max = format!("ab{}", char::MAX);
        assert_eq!(prefix_right_bound(&with_max), s("ac"));

        let with_two_max = format!("ab{}{}", char::MAX, char::MAX);
        assert_eq!(prefix_right_bound(&with_two_max), s("ac"));

        let with_three_max = format!("aa{}{}{}", char::MAX, char::MAX, char::MAX);
        assert_eq!(prefix_right_bound(&with_three_max), s("ab"));
    }

    #[test]
    fn test_all_max_chars() {
        let all_max = format!("{}{}", char::MAX, char::MAX);
        assert_eq!(prefix_right_bound(&all_max), None);

        let single_max = char::MAX.to_string();
        assert_eq!(prefix_right_bound(&single_max), None);
    }

    #[test]
    fn test_range_query_semantics() {
        // Verify the bound correctly excludes non-matching strings
        let prefix = "foo";
        let bound = prefix_right_bound(prefix).unwrap();
        let bound = bound.as_str();

        // These should be in range [prefix, bound)
        assert!("foo" >= prefix && "foo" < bound);
        assert!("foobar" >= prefix && "foobar" < bound);
        assert!("foo/something" >= prefix && "foo/something" < bound);

        // These should be outside the range
        assert!("fop" >= bound);
        assert!("goo" >= bound);
        assert!("fo" < prefix);
    }
}
