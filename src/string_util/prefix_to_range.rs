use crate::string_util::prefix_right_bound;

/// Converts a prefix to a range that covers all strings starting with the prefix.
///
/// Returns `(start, end)` where:
/// - `start` is the prefix itself
/// - `end` is the exclusive right bound, or `None` if no valid bound exists
///
/// The returned range can be used directly with range queries to match all strings
/// with the given prefix.
///
/// # Examples
///
/// ```
/// use databend_base::string_util::prefix_to_range;
///
/// // Normal case: bounded range
/// assert_eq!(prefix_to_range("foo"), ("foo".to_string(), Some("fop".to_string())));
///
/// // Empty prefix: unbounded range (matches everything)
/// assert_eq!(prefix_to_range(""), ("".to_string(), None));
///
/// // All max chars: unbounded on the right
/// let max_str = char::MAX.to_string();
/// assert_eq!(prefix_to_range(&max_str), (max_str, None));
/// ```
pub fn prefix_to_range(prefix: &str) -> (String, Option<String>) {
    (prefix.to_string(), prefix_right_bound(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_to_range_basic() {
        assert_eq!(
            prefix_to_range("foo"),
            ("foo".to_string(), Some("fop".to_string()))
        );
        assert_eq!(
            prefix_to_range("a"),
            ("a".to_string(), Some("b".to_string()))
        );
    }

    #[test]
    fn test_prefix_to_range_empty() {
        assert_eq!(prefix_to_range(""), ("".to_string(), None));
    }

    #[test]
    fn test_prefix_to_range_unbounded() {
        let max_str = char::MAX.to_string();
        assert_eq!(prefix_to_range(&max_str), (max_str, None));
    }
}
