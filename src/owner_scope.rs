//! Stable, privacy-preserving scope for current-user owner transactions.

use sha2::{Digest, Sha256};

pub const PREFIX: &str = "owner-scope-v1:sha256:";

pub fn from_stable_ids(machine_guid: &str, user_sid: &str) -> String {
    let canonical = format!(
        "owner-scope-v1\0{}\0{}",
        machine_guid.trim().to_ascii_lowercase(),
        user_sid.trim().to_ascii_uppercase()
    );
    let digest = Sha256::digest(canonical.as_bytes());
    format!(
        "{PREFIX}{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

pub fn is_valid(scope: &str) -> bool {
    scope.strip_prefix(PREFIX).is_some_and(|digest| {
        digest.len() == 64
            && digest
                .as_bytes()
                .iter()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_is_stable_and_versioned() {
        let first = from_stable_ids("ABC-123", "S-1-5-21-1000");
        let second = from_stable_ids("abc-123", "s-1-5-21-1000");
        assert_eq!(first, second);
        assert!(is_valid(&first));
        assert!(!first.contains("abc-123"));
        assert!(!first.contains("S-1-5"));
    }

    #[test]
    fn another_windows_user_has_a_different_scope() {
        assert_ne!(
            from_stable_ids("machine", "S-1-5-21-1000"),
            from_stable_ids("machine", "S-1-5-21-1001")
        );
    }

    #[test]
    fn legacy_machine_hash_is_not_an_owner_scope() {
        assert!(!is_valid("0123456789abcdef"));
        assert!(!is_valid("owner-scope-v2:sha256:00"));
    }
}
