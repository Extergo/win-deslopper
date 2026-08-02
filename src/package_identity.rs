//! Exact, versioned package identity rules. Names are never matched fuzzily.

use crate::platform::ComponentId;

pub const IDENTITY_RULE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityRole {
    Canonical,
    Legacy,
    Successor,
    RequiredCompanion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageIdentityRule {
    pub name: &'static str,
    pub role: IdentityRole,
    pub minimum_build: u32,
    pub maximum_build: Option<u32>,
    pub mutually_exclusive_group: Option<&'static str>,
}

pub fn rules(id: ComponentId) -> &'static [PackageIdentityRule] {
    use IdentityRole::*;
    match id {
        ComponentId::Microsoft365Copilot => &[
            PackageIdentityRule {
                name: "Microsoft.MicrosoftOfficeHub",
                role: Legacy,
                minimum_build: 22000,
                maximum_build: None,
                mutually_exclusive_group: Some("m365-hub"),
            },
            PackageIdentityRule {
                name: "Microsoft.Microsoft365Copilot",
                role: Successor,
                minimum_build: 26100,
                maximum_build: None,
                mutually_exclusive_group: Some("m365-hub"),
            },
        ],
        ComponentId::PhoneLink => &[PackageIdentityRule {
            name: "Microsoft.YourPhone",
            role: Canonical,
            minimum_build: 19041,
            maximum_build: None,
            mutually_exclusive_group: None,
        }],
        ComponentId::ConsumerCopilot => &[PackageIdentityRule {
            name: "Microsoft.Copilot",
            role: Canonical,
            minimum_build: 22621,
            maximum_build: None,
            mutually_exclusive_group: None,
        }],
        ComponentId::PersonalTeamsChat => &[
            PackageIdentityRule {
                name: "MicrosoftTeamsClassic",
                role: Legacy,
                minimum_build: 22000,
                maximum_build: Some(22631),
                mutually_exclusive_group: Some("personal-teams"),
            },
            PackageIdentityRule {
                name: "MicrosoftTeams",
                role: Successor,
                minimum_build: 22000,
                maximum_build: None,
                mutually_exclusive_group: Some("personal-teams"),
            },
        ],
        ComponentId::Clipchamp => &[PackageIdentityRule {
            name: "Clipchamp.Clipchamp",
            role: Canonical,
            minimum_build: 22000,
            maximum_build: None,
            mutually_exclusive_group: None,
        }],
        ComponentId::NewsWeather => &[
            PackageIdentityRule {
                name: "Microsoft.BingWeather",
                role: RequiredCompanion,
                minimum_build: 19041,
                maximum_build: None,
                mutually_exclusive_group: None,
            },
            PackageIdentityRule {
                name: "Microsoft.BingNews",
                role: RequiredCompanion,
                minimum_build: 19041,
                maximum_build: None,
                mutually_exclusive_group: None,
            },
        ],
        ComponentId::Solitaire => &[PackageIdentityRule {
            name: "Microsoft.MicrosoftSolitaireCollection",
            role: Canonical,
            minimum_build: 19041,
            maximum_build: None,
            mutually_exclusive_group: None,
        }],
        _ => &[],
    }
}

pub fn exact_match(id: ComponentId, package_name: &str, build: u32) -> bool {
    rules(id).iter().any(|rule| {
        rule.name.eq_ignore_ascii_case(package_name)
            && build >= rule.minimum_build
            && rule.maximum_build.is_none_or(|maximum| build <= maximum)
    })
}

pub fn identity_migrated(id: ComponentId, before: &[String], after: &[String]) -> bool {
    let rules = rules(id);
    let before_legacy = rules.iter().any(|rule| {
        rule.role == IdentityRole::Legacy
            && before
                .iter()
                .any(|name| name.eq_ignore_ascii_case(rule.name))
    });
    let after_successor = rules.iter().any(|rule| {
        rule.role == IdentityRole::Successor
            && after
                .iter()
                .any(|name| name.eq_ignore_ascii_case(rule.name))
    });
    before_legacy && after_successor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_fuzzy_package_is_never_matched() {
        assert!(!exact_match(
            ComponentId::ConsumerCopilot,
            "Contoso.CopilotHelper",
            26100
        ));
    }

    #[test]
    fn known_legacy_to_successor_change_is_a_migration() {
        assert!(identity_migrated(
            ComponentId::PersonalTeamsChat,
            &["MicrosoftTeamsClassic".into()],
            &["MicrosoftTeams".into()]
        ));
    }

    #[test]
    fn news_and_weather_is_multi_identity() {
        assert_eq!(rules(ComponentId::NewsWeather).len(), 2);
    }
}
