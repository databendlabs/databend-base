/// Global unique ID generator.
///
/// Generates random base62-encoded strings from UUIDv4.
/// Output is 21-22 alphanumeric characters (0-9, a-z, A-Z).
///
/// # Example
/// ```
/// use databend_base::uniq_id::GlobalUniq;
///
/// let id = GlobalUniq::unique();
/// assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
/// ```
pub struct GlobalUniq;

impl GlobalUniq {
    pub fn unique() -> String {
        let mut uuid = uuid::Uuid::new_v4().as_u128();
        let mut buf = Vec::with_capacity(22);

        loop {
            let m = (uuid % 62) as u8;
            uuid /= 62;

            match m {
                0..=9 => buf.push((b'0' + m) as char),
                10..=35 => buf.push((b'a' + (m - 10)) as char),
                36..=61 => buf.push((b'A' + (m - 36)) as char),
                unreachable => unreachable!("Unreachable branch m = {}", unreachable),
            }

            if uuid == 0 {
                return buf.iter().collect();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniq_generates_valid_base62() {
        let id = GlobalUniq::unique();
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(!id.is_empty());
    }

    #[test]
    fn test_uniq_generates_different_values() {
        let a = GlobalUniq::unique();
        let b = GlobalUniq::unique();
        assert_ne!(a, b);
    }
}
