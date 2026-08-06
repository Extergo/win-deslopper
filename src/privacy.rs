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

pub fn redact_diagnostic(value: &str) -> String {
    let mut output = redact(value);
    for (name, label) in [
        ("USERNAME", "[redacted-user]"),
        ("COMPUTERNAME", "[redacted-device]"),
        ("USERDOMAIN", "[redacted-domain]"),
    ] {
        if let Ok(candidate) = std::env::var(name)
            && !candidate.is_empty()
        {
            output = output.replace(&candidate, label);
        }
    }
    output
        .split_whitespace()
        .map(|word| {
            if word.contains('@') && word.contains('.') {
                "[redacted-email]".to_owned()
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

    #[test]
    fn diagnostic_redaction_removes_account_device_and_email_markers() {
        let mut input = "contact person@example.test".to_owned();
        if let Ok(user) = std::env::var("USERNAME") {
            input.push_str(&format!(" user={user}"));
        }
        if let Ok(device) = std::env::var("COMPUTERNAME") {
            input.push_str(&format!(" device={device}"));
        }
        let redacted = redact_diagnostic(&input);
        assert!(!redacted.contains("person@example.test"));
        if let Ok(user) = std::env::var("USERNAME") {
            assert!(!redacted.contains(&user));
        }
        if let Ok(device) = std::env::var("COMPUTERNAME") {
            assert!(!redacted.contains(&device));
        }
    }
}
