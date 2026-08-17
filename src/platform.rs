//! Research-backed platform domain types. This module is deliberately pure: it
//! contains no registry paths, shell commands, elevation, or UI concerns.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentId {
    Onedrive,
    Microsoft365Copilot,
    PhoneLink,
    ConsumerCopilot,
    WidgetsPlatform,
    ConsumerExperiences,
    WelcomeExperience,
    TipsSuggestions,
    LockScreenSuggestions,
    StartRecommendations,
    NotificationSuggestions,
    SettingsSuggestedContent,
    SearchWebResults,
    SearchHighlights,
    TaskbarWidgets,
    TaskbarTaskView,
    TaskbarSearch,
    PersonalTeamsChat,
    Clipchamp,
    NewsWeather,
    Solitaire,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Risk {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SupportStatus {
    Support,
    MonitorOnly,
    ResearchFurther,
    DoNotSupport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Authority {
    Firmware,
    Oem,
    DomainPolicy,
    Mdm,
    LocalPolicy,
    SecurityProduct,
    WindowsServicing,
    Deslopper,
    User,
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityConfidence {
    Confirmed,
    Strong,
    Moderate,
    Weak,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityAttribution {
    pub authority: Authority,
    pub confidence: AuthorityConfidence,
    pub exact_source_proven: bool,
    pub evidence: Vec<String>,
    pub alternatives: Vec<Authority>,
}

impl Default for AuthorityAttribution {
    fn default() -> Self {
        Self {
            authority: Authority::Unknown,
            confidence: AuthorityConfidence::Unknown,
            exact_source_proven: false,
            evidence: Vec::new(),
            alternatives: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityStatus {
    Applicable,
    PartiallyApplicable,
    NotApplicable,
    UnsupportedEdition,
    UnsupportedBuild,
    Deprecated,
    Removed,
    MissingPrerequisite,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicabilityResult {
    pub status: ApplicabilityStatus,
    pub reason: String,
    pub policy_supported: bool,
    pub preference_supported: bool,
    pub representation: String,
    pub rule_version: u32,
    pub source: String,
    pub source_reviewed_at: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectorStatus {
    Successful,
    Unknown,
    Failed,
    Cancelled,
    #[default]
    NotRun,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageRegistrationState {
    Present,
    Absent,
    PermissionLimited,
    QueryFailed,
    NotApplicable,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageProvisioningState {
    Provisioned,
    NotProvisioned,
    PermissionLimited,
    QueryFailed,
    NotApplicable,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageCompleteness {
    Complete,
    CurrentUserOnly,
    RegistrationCompleteProvisioningUnknown,
    ProvisioningCompleteAllUsersUnknown,
    PermissionLimited,
    Failed,
    NotApplicable,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilesOnDemandState {
    NotInstalled,
    InstalledUnlinked,
    SupportedEnabled,
    SupportedDisabled,
    MixedOrPerRootUnknown,
    PolicyEnforced,
    Unsupported,
    #[default]
    DetectionIncomplete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageObservation {
    pub component_id: ComponentId,
    pub package_family_name: Option<String>,
    pub package_full_name: Option<String>,
    pub package_name: String,
    pub version: Option<String>,
    pub architecture: Option<String>,
    pub publisher_id: Option<String>,
    pub resource_id: Option<String>,
    pub provisioned_package_full_name: Option<String>,
    pub current_user: PackageRegistrationState,
    pub other_users: PackageRegistrationState,
    pub provisioning: PackageProvisioningState,
    pub framework: bool,
    pub resource_package: bool,
    pub bundle: bool,
    pub non_removable: bool,
    pub install_location_present: Option<bool>,
    pub dependencies: Vec<String>,
    pub source_query_completeness: PackageCompleteness,
    pub permission_status: String,
    pub observed_at: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlPrecedence {
    pub documented_default: Option<String>,
    pub user_preference: Option<String>,
    pub local_policy: Option<String>,
    pub domain_policy: Option<String>,
    pub mdm_policy: Option<String>,
    pub effective_state: Option<String>,
    pub effective_authority: AuthorityAttribution,
    pub conflicting_evidence: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RestartRequirement {
    None,
    Shell,
    Application,
    SignOut,
    Reboot,
    MultiStep,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum State {
    Policy {
        configured: bool,
        enabled: bool,
    },
    UserPreference {
        enabled: bool,
    },
    Package {
        current_user: PackageRegistrationState,
        all_users: PackageRegistrationState,
        provisioned: PackageProvisioningState,
        version: Option<String>,
    },
    OneDrive {
        installed: bool,
        version: Option<String>,
        running: bool,
        startup: bool,
        personal_account: bool,
        work_account: bool,
        sync_root_count: u32,
        desktop_redirected: bool,
        documents_redirected: bool,
        pictures_redirected: bool,
        kfm_policy_configured: bool,
        files_on_demand: FilesOnDemandState,
        partial: bool,
    },
    Unsupported {
        reason: String,
    },
    Managed {
        authority: Authority,
        source: Option<String>,
    },
    Unknown {
        error: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    #[serde(alias = "product_name")]
    pub product_name: String,
    pub edition: String,
    pub build: u32,
    #[serde(alias = "display_version")]
    pub display_version: String,
    #[serde(alias = "update_build_revision")]
    pub update_build_revision: Option<u32>,
    pub architecture: String,
    #[serde(alias = "device_name")]
    pub device_name: Option<String>,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(alias = "user_sid")]
    pub user_sid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_scope_id: Option<String>,
    pub elevated: bool,
    #[serde(alias = "domain_joined")]
    pub domain_joined: Option<bool>,
    #[serde(default, alias = "entra_joined")]
    pub entra_joined: Option<bool>,
    #[serde(alias = "workplace_joined")]
    pub workplace_joined: Option<bool>,
    #[serde(alias = "mdm_enrolled")]
    pub mdm_enrolled: Option<bool>,
    #[serde(alias = "is_windows_11")]
    pub is_windows_11: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    #[serde(default)]
    #[serde(alias = "query_id")]
    pub query_id: String,
    pub source: String,
    pub detail: String,
    pub confidence: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
    #[serde(alias = "component_id")]
    pub component_id: ComponentId,
    pub current: State,
    pub authority: Authority,
    pub platform: PlatformInfo,
    pub applicable: bool,
    #[serde(default)]
    pub applicability: ApplicabilityResult,
    pub evidence: Vec<Evidence>,
    #[serde(alias = "detected_at")]
    pub detected_at: String,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    #[serde(alias = "package_identities")]
    pub package_identities: Vec<String>,
    #[serde(alias = "policy_state")]
    pub policy_state: Option<String>,
    #[serde(alias = "preference_state")]
    pub preference_state: Option<String>,
    #[serde(alias = "provisioning_state")]
    pub provisioning_state: Option<String>,
    #[serde(default)]
    #[serde(alias = "detector_status")]
    pub detector_status: DetectorStatus,
    #[serde(default)]
    #[serde(alias = "authority_attribution")]
    pub authority_attribution: AuthorityAttribution,
    #[serde(default)]
    #[serde(alias = "control_precedence")]
    pub control_precedence: ControlPrecedence,
    #[serde(default)]
    #[serde(alias = "package_completeness")]
    pub package_completeness: PackageCompleteness,
    #[serde(default)]
    pub packages: Vec<PackageObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ComponentDefinition {
    pub id: ComponentId,
    pub name: &'static str,
    pub category: &'static str,
    pub purpose: &'static str,
    pub benefit: &'static str,
    pub risk: Risk,
    pub support: SupportStatus,
    pub privileges: &'static str,
    pub gaming_notes: &'static str,
    pub enterprise_notes: &'static str,
    pub dependencies: &'static [&'static str],
    pub conflicts: &'static [&'static str],
    pub detection: &'static str,
    pub configuration: &'static str,
    pub rollback: &'static str,
    pub drift_likelihood: &'static str,
    pub drift_causes: &'static [&'static str],
    pub auto_reapply: bool,
    pub approval_required: bool,
    pub restart: RestartRequirement,
    pub documentation: &'static str,
    pub is_package: bool,
}

impl ComponentId {
    pub fn key(self) -> &'static str {
        match self {
            Self::Onedrive => "onedrive",
            Self::Microsoft365Copilot => "microsoft_365_copilot",
            Self::PhoneLink => "phone_link",
            Self::ConsumerCopilot => "consumer_copilot",
            Self::WidgetsPlatform => "widgets_platform",
            Self::ConsumerExperiences => "consumer_experiences",
            Self::WelcomeExperience => "welcome_experience",
            Self::TipsSuggestions => "tips_suggestions",
            Self::LockScreenSuggestions => "lock_screen_suggestions",
            Self::StartRecommendations => "start_recommendations",
            Self::NotificationSuggestions => "notification_suggestions",
            Self::SettingsSuggestedContent => "settings_suggested_content",
            Self::SearchWebResults => "search_web_results",
            Self::SearchHighlights => "search_highlights",
            Self::TaskbarWidgets => "taskbar_widgets",
            Self::TaskbarTaskView => "taskbar_task_view",
            Self::TaskbarSearch => "taskbar_search",
            Self::PersonalTeamsChat => "personal_teams_chat",
            Self::Clipchamp => "clipchamp",
            Self::NewsWeather => "news_weather",
            Self::Solitaire => "solitaire",
        }
    }
}

macro_rules! v1 {
    ($id:ident, $name:literal, $category:literal, $risk:ident, $purpose:literal, $config:literal, $rollback:literal, $docs:literal) => {
        ComponentDefinition {
            id: ComponentId::$id,
            name: $name,
            category: $category,
            purpose: $purpose,
            benefit: "User-reviewed control over a documented Windows surface.",
            risk: Risk::$risk,
            support: SupportStatus::Support,
            privileges: "User or documented policy scope",
            gaming_notes: "Review gaming and shell dependencies before changing.",
            enterprise_notes: "Do not change when domain or MDM authority is detected.",
            dependencies: &[],
            conflicts: &[],
            detection: "Read-only supported Windows policy, preference, or package evidence",
            configuration: $config,
            rollback: $rollback,
            drift_likelihood: "Medium",
            drift_causes: &["Windows servicing", "user action", "management policy"],
            auto_reapply: false,
            approval_required: true,
            restart: RestartRequirement::None,
            documentation: $docs,
            is_package: matches!(
                ComponentId::$id,
                ComponentId::Microsoft365Copilot
                    | ComponentId::PhoneLink
                    | ComponentId::ConsumerCopilot
                    | ComponentId::PersonalTeamsChat
                    | ComponentId::Clipchamp
                    | ComponentId::NewsWeather
                    | ComponentId::Solitaire
            ),
        }
    };
}

pub fn v1_catalogue() -> Vec<ComponentDefinition> {
    vec![
        v1!(
            Onedrive,
            "OneDrive",
            "Cloud integration",
            Medium,
            "Syncs files and integrates with Explorer.",
            "Separate reviewed startup, account, package, and KFM workflows.",
            "Restore captured state; KFM rollback is partial and never automatic.",
            "https://learn.microsoft.com/windows/client-management/mdm/onedrive-csp"
        ),
        v1!(
            Microsoft365Copilot,
            "Microsoft 365 Copilot app",
            "Application",
            Low,
            "Provides the Microsoft 365 Copilot application.",
            "Remove exact supported package identity after reinstall-path validation.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/microsoft-365"
        ),
        v1!(
            PhoneLink,
            "Phone Link",
            "Application",
            Low,
            "Connects a phone to Windows.",
            "Remove exact current-user package after dependency and management checks.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/phone-link"
        ),
        v1!(
            ConsumerCopilot,
            "Microsoft Copilot app",
            "Assistant",
            Low,
            "Provides the consumer Copilot application.",
            "Remove exact supported package identity after review.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/windows"
        ),
        v1!(
            WidgetsPlatform,
            "Widgets platform",
            "Shell experience",
            Safe,
            "Provides the Windows Widgets surface.",
            "Use documented policy or preference only; never remove Web Experience Pack.",
            "Restore prior policy or preference state.",
            "https://learn.microsoft.com/windows/configuration/organization-configuration/configure-windows-spotlight"
        ),
        v1!(
            ConsumerExperiences,
            "Microsoft consumer experiences",
            "Recommendations",
            Safe,
            "Controls documented consumer suggestions.",
            "Set documented policy where supported by edition.",
            "Restore prior policy including Not Configured.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience"
        ),
        v1!(
            WelcomeExperience,
            "Welcome / finish setting up",
            "Recommendations",
            Safe,
            "Controls setup suggestions after sign-in.",
            "Set documented preference or policy.",
            "Restore captured preference.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience"
        ),
        v1!(
            TipsSuggestions,
            "Tips and suggestions",
            "Recommendations",
            Safe,
            "Controls Windows tips and suggestions.",
            "Set documented preference or policy.",
            "Restore captured preference.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience"
        ),
        v1!(
            LockScreenSuggestions,
            "Lock-screen suggestions",
            "Personalization",
            Safe,
            "Controls Spotlight suggestions on the lock screen.",
            "Set documented personalization preference.",
            "Restore captured preference.",
            "https://learn.microsoft.com/windows/configuration/organization-configuration/configure-windows-spotlight"
        ),
        v1!(
            StartRecommendations,
            "Start recommendations",
            "Shell experience",
            Low,
            "Controls recommendation surfaces in Start.",
            "Change only the documented recommendation class.",
            "Restore captured policy/preference.",
            "https://learn.microsoft.com/windows/configuration/start/policy-settings"
        ),
        v1!(
            NotificationSuggestions,
            "Notification suggestions",
            "Recommendations",
            Safe,
            "Controls suggested notifications.",
            "Set documented preference.",
            "Restore captured preference.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience"
        ),
        v1!(
            SettingsSuggestedContent,
            "Suggested content in Settings",
            "Recommendations",
            Safe,
            "Controls suggested content in Settings.",
            "Set documented preference or policy.",
            "Restore captured preference.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-experience"
        ),
        v1!(
            SearchWebResults,
            "Web/Bing results in Search",
            "Search",
            Low,
            "Controls web results in Windows Search.",
            "Set documented Search policy.",
            "Restore previous policy state.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-search"
        ),
        v1!(
            SearchHighlights,
            "Search highlights",
            "Search",
            Safe,
            "Controls Search highlights.",
            "Set documented Search policy.",
            "Restore previous policy state.",
            "https://learn.microsoft.com/windows/client-management/mdm/policy-csp-search"
        ),
        v1!(
            TaskbarWidgets,
            "Taskbar Widgets button",
            "Taskbar",
            Safe,
            "Controls the Widgets taskbar entry point.",
            "Set documented taskbar preference.",
            "Restore captured preference.",
            "https://learn.microsoft.com/windows/configuration/taskbar/policy-settings"
        ),
        v1!(
            TaskbarTaskView,
            "Taskbar Task View button",
            "Taskbar",
            Safe,
            "Controls the Task View taskbar entry point.",
            "Set the current-user Task View preference after policy checks.",
            "Restore the exact captured preference.",
            "https://learn.microsoft.com/windows/configuration/taskbar/policy-settings"
        ),
        v1!(
            TaskbarSearch,
            "Taskbar Search button/box",
            "Taskbar",
            Safe,
            "Controls taskbar Search presentation.",
            "Set documented taskbar Search presentation.",
            "Restore captured presentation.",
            "https://learn.microsoft.com/windows/configuration/taskbar/policy-settings"
        ),
        v1!(
            PersonalTeamsChat,
            "Personal Teams / legacy Chat",
            "Application",
            Low,
            "Controls the personal Teams/Chat entry point.",
            "Remove only exact personal package identity after checks.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/teams"
        ),
        v1!(
            Clipchamp,
            "Clipchamp",
            "Application",
            Safe,
            "Provides the Clipchamp application.",
            "Remove exact supported package identity after review.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/clipchamp"
        ),
        v1!(
            NewsWeather,
            "News and Weather apps",
            "Application",
            Safe,
            "Provides news and weather applications.",
            "Remove exact package identities after dependency checks.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/windows"
        ),
        v1!(
            Solitaire,
            "Microsoft Solitaire Collection",
            "Application",
            Safe,
            "Provides the Solitaire application.",
            "Remove exact package identity after review.",
            "Reinstall from a known supported source.",
            "https://support.microsoft.com/windows"
        ),
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriftKind {
    Compliant,
    Preference,
    Policy,
    PackagePresence,
    PackageProvisioning,
    Dependency,
    Availability,
    ManagementAuthority,
    SecurityPosture,
    NormalServicing,
    PackageIdentityMigration,
    DetectionUncertainty,
    ResetOrReinstall,
    Unknown,
}

pub fn drift_kind_keys() -> Vec<String> {
    use DriftKind::*;
    [
        Compliant,
        Preference,
        Policy,
        PackagePresence,
        PackageProvisioning,
        Dependency,
        Availability,
        ManagementAuthority,
        SecurityPosture,
        NormalServicing,
        PackageIdentityMigration,
        DetectionUncertainty,
        ResetOrReinstall,
        Unknown,
    ]
    .into_iter()
    .map(|kind| format!("{kind:?}"))
    .collect()
}

pub fn classify_drift(before: &State, after: &State, desired: &State) -> Option<DriftKind> {
    if before == after {
        return None;
    }
    if matches!(after, State::Unknown { .. }) {
        return Some(DriftKind::DetectionUncertainty);
    }
    match (before, after, desired) {
        (State::Package { provisioned: a, .. }, State::Package { provisioned: b, .. }, _)
            if a != b =>
        {
            Some(DriftKind::PackageProvisioning)
        }
        (
            State::Package {
                current_user: a,
                all_users: aa,
                ..
            },
            State::Package {
                current_user: b,
                all_users: bb,
                ..
            },
            _,
        ) if a != b || aa != bb => Some(DriftKind::PackagePresence),
        (
            State::Package {
                version: Some(a), ..
            },
            State::Package {
                version: Some(b), ..
            },
            _,
        ) if a != b => Some(DriftKind::NormalServicing),
        (State::Package { .. }, State::Package { .. }, State::Package { .. }) => {
            Some(DriftKind::PackagePresence)
        }
        (State::Managed { .. }, _, _) | (_, State::Managed { .. }, _) => {
            Some(DriftKind::ManagementAuthority)
        }
        (State::Policy { .. }, State::Policy { .. }, _) => Some(DriftKind::Policy),
        _ => Some(DriftKind::Preference),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(version: &str, provisioned: bool) -> State {
        State::Package {
            current_user: PackageRegistrationState::Present,
            all_users: PackageRegistrationState::Present,
            provisioned: if provisioned {
                PackageProvisioningState::Provisioned
            } else {
                PackageProvisioningState::NotProvisioned
            },
            version: Some(version.to_owned()),
        }
    }

    #[test]
    fn roadmap_contains_exactly_twenty_one_components() {
        let catalogue = v1_catalogue();
        assert_eq!(catalogue.len(), 21);
        assert!(
            catalogue
                .iter()
                .all(|item| item.support == SupportStatus::Support)
        );
        assert!(catalogue.iter().all(|item| item.approval_required));
    }

    #[test]
    fn package_version_change_is_normal_servicing() {
        let before = package("1.0.0", true);
        let after = package("1.1.0", true);
        assert_eq!(
            classify_drift(&before, &after, &before),
            Some(DriftKind::NormalServicing)
        );
    }

    #[test]
    fn provisioning_change_is_not_current_user_presence_drift() {
        let before = package("1.0.0", false);
        let after = package("1.0.0", true);
        assert_eq!(
            classify_drift(&before, &after, &before),
            Some(DriftKind::PackageProvisioning)
        );
    }
}
