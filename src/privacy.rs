//! Redaction applied before evidence is persisted or shown to the frontend.

pub fn redact(value: &str) -> String {
    let mut output = value.to_owned();
    for (name, label) in [
        ("USERPROFILE", "%USERPROFILE%"),
        ("LOCALAPPDATA", "%LOCALAPPDATA%"),
        ("APPDATA", "%APPDATA%"),
    ] {
        if let Ok(path) = std::env::var(name)
            && !path.is_empty()
        {
            output = output
                .replace(&path, label)
                .replace(&path.replace('\\', "/"), label);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_evidence_is_unchanged() {
        assert_eq!(
            redact("No policy value was found"),
            "No policy value was found"
        );
    }
    #[test]
    fn user_profile_is_redacted_when_available() {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            assert!(!redact(&format!("{profile}\\OneDrive")).contains(&profile));
        }
    }
}
