//! Versioned component/build/edition applicability rules. Detector code consumes
//! these results; TypeScript never reimplements the matrix.

use crate::platform::{ApplicabilityResult, ApplicabilityStatus, ComponentId, PlatformInfo};

pub const RULE_VERSION: u32 = 1;
pub const REVIEW_DATE: &str = "2026-08-01";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicabilityRule {
    pub component_id: ComponentId,
    pub minimum_build: u32,
    pub maximum_build: Option<u32>,
    pub windows_10: bool,
    pub windows_11: bool,
    pub home: bool,
    pub pro: bool,
    pub enterprise: bool,
    pub education: bool,
    pub policy_home: bool,
    pub policy_pro: bool,
    pub policy_enterprise: bool,
    pub policy_education: bool,
    pub preference_supported: bool,
    pub deprecated: bool,
    pub removed: bool,
    pub prerequisite: Option<&'static str>,
    pub representation: &'static str,
    pub source: &'static str,
}

fn rule(
    component_id: ComponentId,
    minimum_build: u32,
    windows_10: bool,
    representation: &'static str,
    source: &'static str,
) -> ApplicabilityRule {
    ApplicabilityRule {
        component_id,
        minimum_build,
        maximum_build: None,
        windows_10,
        windows_11: true,
        home: true,
        pro: true,
        enterprise: true,
        education: true,
        policy_home: true,
        policy_pro: true,
        policy_enterprise: true,
        policy_education: true,
        preference_supported: true,
        deprecated: false,
        removed: false,
        prerequisite: None,
        representation,
        source,
    }
}

pub fn component_rule(id: ComponentId) -> ApplicabilityRule {
    use ComponentId::*;
    match id {
        Onedrive => rule(
            id,
            19041,
            true,
            "OneDrive client and documented policy",
            "https://learn.microsoft.com/sharepoint/use-group-policy",
        ),
        Microsoft365Copilot => rule(
            id,
            22000,
            false,
            "MSIX package identity",
            "https://support.microsoft.com/microsoft-365",
        ),
        PhoneLink => rule(
            id,
            19041,
            true,
            "MSIX package identity",
            "https://support.microsoft.com/topic/phone-link-requirements-and-setup-cd2a1ee7-75a7-66a6-9d4e-bf22e735f9e3",
        ),
        ConsumerCopilot => rule(
            id,
            22621,
            false,
            "Standalone Copilot MSIX identity; representation varies by servicing",
            "https://support.microsoft.com/windows/welcome-to-copilot-on-windows-675708af-8c16-4675-afeb-85a5a476ccb0",
        ),
        WidgetsPlatform => rule(
            id,
            22000,
            false,
            "Windows Widgets policy plus taskbar preference",
            "https://learn.microsoft.com/windows/configuration/organization-configuration/configure-widgets",
        ),
        ConsumerExperiences => {
            let mut value = rule(
                id,
                19041,
                true,
                "CloudContent policy",
                "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience#allowwindowsspotlight",
            );
            value.policy_home = false;
            value.policy_pro = false;
            value
        }
        WelcomeExperience
        | TipsSuggestions
        | NotificationSuggestions
        | SettingsSuggestedContent => rule(
            id,
            19041,
            true,
            "CloudContent policy and current-user ContentDeliveryManager preference",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience",
        ),
        LockScreenSuggestions => rule(
            id,
            19041,
            true,
            "Windows Spotlight policy and lock-screen preference",
            "https://learn.microsoft.com/windows/configuration/organization-configuration/configure-windows-spotlight",
        ),
        StartRecommendations => rule(
            id,
            26100,
            false,
            "Start policy; HideRecommendedSection on 24H2 requires the servicing level that introduced the policy",
            "https://learn.microsoft.com/windows/configuration/start/policy-settings",
        ),
        SearchWebResults => {
            let mut value = rule(
                id,
                19041,
                true,
                "Windows Search DoNotUseWebResults policy",
                "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-search#donotusewebresults",
            );
            value.policy_home = false;
            value.policy_pro = false;
            value.preference_supported = false;
            value
        }
        SearchHighlights => rule(
            id,
            19041,
            true,
            "Windows Search AllowSearchHighlights policy",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-search#allowsearchhighlights",
        ),
        TaskbarWidgets => rule(
            id,
            22000,
            false,
            "Current-user Explorer taskbar preference",
            "https://learn.microsoft.com/windows/configuration/taskbar/policy-settings",
        ),
        TaskbarTaskView => rule(
            id,
            22000,
            true,
            "Current-user Explorer Task View preference",
            "https://learn.microsoft.com/windows/configuration/taskbar/policy-settings",
        ),
        TaskbarSearch => rule(
            id,
            19041,
            true,
            "Current-user Search taskbar presentation",
            "https://learn.microsoft.com/windows/configuration/taskbar/policy-settings",
        ),
        PersonalTeamsChat => {
            let mut value = rule(
                id,
                22000,
                false,
                "Legacy Chat and successor Teams MSIX identities",
                "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience#configurechaticon",
            );
            value.representation = "Chat representation retired on newer Windows 11 builds; Teams package identity remains observational";
            value
        }
        Clipchamp => rule(
            id,
            22000,
            false,
            "MSIX package identity",
            "https://support.microsoft.com/clipchamp",
        ),
        NewsWeather => rule(
            id,
            19041,
            true,
            "Bing News and Bing Weather exact MSIX identities",
            "https://support.microsoft.com/windows/apps-for-the-web-1a6f84b7-1746-45f0-a294-2b0059b7a1f4",
        ),
        Solitaire => rule(
            id,
            19041,
            true,
            "Microsoft Solitaire Collection exact MSIX identity",
            "https://support.microsoft.com/windows",
        ),
    }
}

fn edition(info: &PlatformInfo) -> &'static str {
    let lower = info.edition.to_ascii_lowercase();
    if lower.contains("education") {
        "education"
    } else if lower.contains("enterprise") {
        "enterprise"
    } else if lower.contains("professional") || lower.contains(" pro") {
        "pro"
    } else if lower.contains("home") || lower.contains("core") {
        "home"
    } else {
        "unknown"
    }
}

pub fn evaluate(id: ComponentId, info: &PlatformInfo) -> ApplicabilityResult {
    let rule = component_rule(id);
    let policy_supported = match edition(info) {
        "home" => rule.policy_home,
        "pro" => rule.policy_pro,
        "enterprise" => rule.policy_enterprise,
        "education" => rule.policy_education,
        _ => false,
    };
    let edition_supported = match edition(info) {
        "home" => rule.home,
        "pro" => rule.pro,
        "enterprise" => rule.enterprise,
        "education" => rule.education,
        _ => false,
    };
    let windows_generation_supported = if info.build >= 22000 {
        rule.windows_11
    } else {
        rule.windows_10
    };
    let (status, reason) = if info.build == 0 {
        (
            ApplicabilityStatus::Unknown,
            "Windows build was not detected",
        )
    } else if rule.removed {
        (
            ApplicabilityStatus::Removed,
            "The component representation was removed",
        )
    } else if rule.deprecated {
        (
            ApplicabilityStatus::Deprecated,
            "The component representation is deprecated",
        )
    } else if info.build < rule.minimum_build
        || rule
            .maximum_build
            .is_some_and(|maximum| info.build > maximum)
        || !windows_generation_supported
    {
        (
            ApplicabilityStatus::UnsupportedBuild,
            "The rule does not apply to this Windows build",
        )
    } else if id == ComponentId::StartRecommendations
        && info.build == 26100
        && info.update_build_revision.is_some_and(|ubr| ubr < 4770)
    {
        (
            ApplicabilityStatus::MissingPrerequisite,
            "HideRecommendedSection requires Windows 11 24H2 OS build 26100.4770 or a later cumulative update",
        )
    } else if !edition_supported || (!policy_supported && !rule.preference_supported) {
        (
            ApplicabilityStatus::UnsupportedEdition,
            "The component is unsupported on this edition",
        )
    } else if !policy_supported && rule.preference_supported {
        (
            ApplicabilityStatus::PartiallyApplicable,
            "The preference is available but the policy surface is unsupported or unverified on this edition",
        )
    } else {
        (
            ApplicabilityStatus::Applicable,
            "The component representation applies to this build and edition",
        )
    };
    ApplicabilityResult {
        status,
        reason: reason.into(),
        policy_supported,
        preference_supported: rule.preference_supported,
        representation: rule.representation.into(),
        rule_version: RULE_VERSION,
        source: rule.source.into(),
        source_reviewed_at: REVIEW_DATE.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspection::default_platform;

    fn platform(edition: &str, build: u32) -> PlatformInfo {
        PlatformInfo {
            edition: edition.into(),
            build,
            is_windows_11: Some(build >= 22000),
            ..default_platform()
        }
    }

    #[test]
    fn home_consumer_experience_policy_is_partial() {
        let result = evaluate(
            ComponentId::ConsumerExperiences,
            &platform("Windows 11 Home", 26100),
        );
        assert_eq!(result.status, ApplicabilityStatus::PartiallyApplicable);
        assert!(!result.policy_supported);
    }

    #[test]
    fn enterprise_consumer_experience_policy_is_supported() {
        let result = evaluate(
            ComponentId::ConsumerExperiences,
            &platform("Windows 11 Enterprise", 26100),
        );
        assert_eq!(result.status, ApplicabilityStatus::Applicable);
        assert!(result.policy_supported);
    }

    #[test]
    fn build_too_old_is_not_reported_disabled() {
        let result = evaluate(
            ComponentId::TaskbarWidgets,
            &platform("Windows 10 Pro", 19045),
        );
        assert_eq!(result.status, ApplicabilityStatus::UnsupportedBuild);
    }
}
