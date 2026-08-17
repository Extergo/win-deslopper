//! Fixed current-user ContentDeliveryManager cleanup settings for Owner Mode M3.

use super::{
    HandlerError, MutationBackend, OperationHandler, capture, expected, require_user_authority,
    verify_exact, verify_target,
};
use crate::mutation::{
    plan::{CapturedRepresentation, CapturedState},
    request::{MutationOperationId, MutationTarget},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupSetting {
    WelcomeExperience,
    TipsSuggestions,
    NotificationSuggestions,
    SettingsSuggestedContent,
}

impl CleanupSetting {
    pub const fn preference_value(self) -> &'static str {
        match self {
            Self::WelcomeExperience => "SubscribedContent-310093Enabled",
            Self::TipsSuggestions => "SoftLandingEnabled",
            Self::NotificationSuggestions => "SubscribedContent-338389Enabled",
            Self::SettingsSuggestedContent => "SubscribedContent-338393Enabled",
        }
    }

    pub const fn policy_value(self) -> &'static str {
        match self {
            Self::WelcomeExperience => "DisableWindowsSpotlightWindowsWelcomeExperience",
            Self::TipsSuggestions => "DisableSoftLanding",
            Self::NotificationSuggestions => "DisableWindowsSpotlightOnActionCenter",
            Self::SettingsSuggestedContent => "DisableWindowsSpotlightOnSettings",
        }
    }
}

pub struct CurrentUserCleanupHandler {
    operation_id: MutationOperationId,
    setting: CleanupSetting,
}

impl CurrentUserCleanupHandler {
    pub const fn new(operation_id: MutationOperationId, setting: CleanupSetting) -> Self {
        Self {
            operation_id,
            setting,
        }
    }
}

impl OperationHandler for CurrentUserCleanupHandler {
    fn operation_id(&self) -> MutationOperationId {
        self.operation_id
    }

    fn inspect_pre_state(
        &self,
        backend: &dyn MutationBackend,
    ) -> Result<CapturedState, HandlerError> {
        capture(
            backend.read_cleanup(self.setting)?,
            None,
            backend.cleanup_externally_managed(self.setting)?,
        )
    }

    fn validate_target(&self, _target: MutationTarget) -> Result<(), HandlerError> {
        Ok(())
    }

    fn apply(
        &self,
        backend: &dyn MutationBackend,
        target: MutationTarget,
    ) -> Result<(), HandlerError> {
        backend.write_cleanup(self.setting, &expected(target))
    }

    fn verify(
        &self,
        backend: &dyn MutationBackend,
        target: MutationTarget,
    ) -> Result<CapturedState, HandlerError> {
        verify_target(self.inspect_pre_state(backend)?, target)
    }

    fn build_rollback(&self, pre_state: &CapturedState) -> CapturedRepresentation {
        pre_state.representation.clone()
    }

    fn rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<(), HandlerError> {
        require_user_authority(pre_state)?;
        backend.write_cleanup(self.setting, &self.build_rollback(pre_state))
    }

    fn verify_rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<CapturedState, HandlerError> {
        verify_exact(self.inspect_pre_state(backend)?, pre_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_metadata_matches_the_scanner_contract() {
        assert_eq!(
            CleanupSetting::WelcomeExperience.preference_value(),
            "SubscribedContent-310093Enabled"
        );
        assert_eq!(
            CleanupSetting::TipsSuggestions.preference_value(),
            "SoftLandingEnabled"
        );
        assert_eq!(
            CleanupSetting::NotificationSuggestions.preference_value(),
            "SubscribedContent-338389Enabled"
        );
        assert_eq!(
            CleanupSetting::SettingsSuggestedContent.preference_value(),
            "SubscribedContent-338393Enabled"
        );
        for setting in [
            CleanupSetting::WelcomeExperience,
            CleanupSetting::TipsSuggestions,
            CleanupSetting::NotificationSuggestions,
            CleanupSetting::SettingsSuggestedContent,
        ] {
            assert!(setting.policy_value().starts_with("Disable"));
        }
    }
}
