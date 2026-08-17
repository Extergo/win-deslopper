use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationOperationId {
    #[serde(rename = "set_taskbar_widgets_visibility")]
    WidgetsVisibility,
    #[serde(rename = "set_taskbar_task_view_visibility")]
    TaskViewVisibility,
    #[serde(rename = "set_taskbar_show_desktop_enabled")]
    ShowDesktopEnabled,
    #[serde(rename = "set_welcome_experience_enabled")]
    WelcomeExperienceEnabled,
    #[serde(rename = "set_tips_suggestions_enabled")]
    TipsSuggestionsEnabled,
    #[serde(rename = "set_notification_suggestions_enabled")]
    NotificationSuggestionsEnabled,
    #[serde(rename = "set_settings_suggested_content_enabled")]
    SettingsSuggestedContentEnabled,
}

impl MutationOperationId {
    #[cfg(any(test, feature = "mutation-alpha"))]
    pub const ALL: [Self; 3] = [
        Self::WidgetsVisibility,
        Self::TaskViewVisibility,
        Self::ShowDesktopEnabled,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::WidgetsVisibility => "set_taskbar_widgets_visibility",
            Self::TaskViewVisibility => "set_taskbar_task_view_visibility",
            Self::ShowDesktopEnabled => "set_taskbar_show_desktop_enabled",
            Self::WelcomeExperienceEnabled => "set_welcome_experience_enabled",
            Self::TipsSuggestionsEnabled => "set_tips_suggestions_enabled",
            Self::NotificationSuggestionsEnabled => "set_notification_suggestions_enabled",
            Self::SettingsSuggestedContentEnabled => "set_settings_suggested_content_enabled",
        }
    }

    pub const fn subject(self) -> MutationSubjectId {
        match self {
            Self::WidgetsVisibility => MutationSubjectId::Widgets,
            Self::TaskViewVisibility => MutationSubjectId::TaskView,
            Self::ShowDesktopEnabled => MutationSubjectId::ShowDesktop,
            Self::WelcomeExperienceEnabled => MutationSubjectId::WelcomeExperience,
            Self::TipsSuggestionsEnabled => MutationSubjectId::TipsSuggestions,
            Self::NotificationSuggestionsEnabled => MutationSubjectId::NotificationSuggestions,
            Self::SettingsSuggestedContentEnabled => MutationSubjectId::SettingsSuggestedContent,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationSubjectId {
    Widgets,
    TaskView,
    ShowDesktop,
    WelcomeExperience,
    TipsSuggestions,
    NotificationSuggestions,
    SettingsSuggestedContent,
}

impl MutationSubjectId {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Widgets => "taskbar_widgets",
            Self::TaskView => "taskbar_task_view",
            Self::ShowDesktop => "taskbar_show_desktop",
            Self::WelcomeExperience => "welcome_experience",
            Self::TipsSuggestions => "tips_suggestions",
            Self::NotificationSuggestions => "notification_suggestions",
            Self::SettingsSuggestedContent => "settings_suggested_content",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationTarget {
    Enabled,
    Disabled,
}

impl MutationTarget {
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanRequest {
    pub operation_id: MutationOperationId,
    pub target: MutationTarget,
    pub source_inspection_id: String,
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalRequest {
    pub plan_id: String,
    pub approval_nonce: String,
    pub acknowledged: bool,
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RollbackRequest {
    pub transaction_id: String,
    pub acknowledged: bool,
    pub allow_conflict: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnerApplyRequest {
    pub operation_id: MutationOperationId,
    pub target: MutationTarget,
    pub source_inspection_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnerUndoRequest {
    pub transaction_id: String,
}

#[cfg(feature = "mutation-alpha")]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaGateStatus {
    pub compiled: bool,
    pub debug_build: bool,
    pub command_line_opt_in: bool,
    pub warning_acknowledged: bool,
    pub warning_text: &'static str,
    pub live_validation: super::live_validation::LiveValidationGateStatus,
    pub available: bool,
    pub reason: String,
}

#[cfg(all(test, feature = "mutation-alpha"))]
mod tests {
    use super::PlanRequest;

    #[test]
    fn plan_request_rejects_multi_step_and_extra_step_shapes() {
        let multi_step = serde_json::json!({
            "steps": [
                {
                    "operationId": "set_taskbar_widgets_visibility",
                    "target": "enabled"
                },
                {
                    "operationId": "set_taskbar_task_view_visibility",
                    "target": "enabled"
                }
            ],
            "sourceInspectionId": "inspection-1"
        });
        assert!(serde_json::from_value::<PlanRequest>(multi_step).is_err());

        let extra_step = serde_json::json!({
            "operationId": "set_taskbar_widgets_visibility",
            "target": "enabled",
            "sourceInspectionId": "inspection-1",
            "extraStep": {
                "operationId": "set_taskbar_show_desktop_enabled",
                "target": "enabled"
            }
        });
        assert!(serde_json::from_value::<PlanRequest>(extra_step).is_err());
    }
}
