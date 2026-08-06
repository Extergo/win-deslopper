mod search_highlights;
mod taskbar_search;
mod taskbar_widgets;
mod windows_store;

use serde::Serialize;

use super::{
    plan::{CapturedRepresentation, CapturedState},
    request::{MutationOperationId, MutationTarget},
};

pub use search_highlights::TaskbarShowDesktopHandler;
pub use taskbar_search::TaskbarTaskViewHandler;
pub use taskbar_widgets::TaskbarWidgetsHandler;
pub use windows_store::WindowsSettingStore;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerErrorKind {
    ReadFailed,
    WriteFailed,
    MissingRepresentation,
    InvalidRepresentation,
    PolicyOverride,
    VerificationMismatch,
    #[cfg(not(windows))]
    UnsupportedPlatform,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HandlerError {
    pub kind: HandlerErrorKind,
    pub summary: String,
}

impl HandlerError {
    pub fn new(kind: HandlerErrorKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
        }
    }
}

pub trait MutationBackend: Send + Sync {
    fn read_widgets(&self) -> Result<CapturedRepresentation, HandlerError>;
    fn widgets_externally_managed(&self) -> Result<bool, HandlerError>;
    fn write_widgets(&self, state: &CapturedRepresentation) -> Result<(), HandlerError>;
    fn read_task_view(&self) -> Result<CapturedRepresentation, HandlerError>;
    fn task_view_externally_managed(&self) -> Result<bool, HandlerError>;
    fn write_task_view(&self, state: &CapturedRepresentation) -> Result<(), HandlerError>;
    fn read_show_desktop(&self) -> Result<CapturedRepresentation, HandlerError>;
    fn show_desktop_externally_managed(&self) -> Result<bool, HandlerError>;
    fn write_show_desktop(&self, state: &CapturedRepresentation) -> Result<(), HandlerError>;
}

pub trait OperationHandler: Send + Sync {
    fn operation_id(&self) -> MutationOperationId;
    fn inspect_pre_state(
        &self,
        backend: &dyn MutationBackend,
    ) -> Result<CapturedState, HandlerError>;
    fn validate_target(&self, target: MutationTarget) -> Result<(), HandlerError>;
    fn apply(
        &self,
        backend: &dyn MutationBackend,
        target: MutationTarget,
    ) -> Result<(), HandlerError>;
    fn verify(
        &self,
        backend: &dyn MutationBackend,
        target: MutationTarget,
    ) -> Result<CapturedState, HandlerError>;
    fn build_rollback(&self, pre_state: &CapturedState) -> CapturedRepresentation;
    fn rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<(), HandlerError>;
    fn verify_rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<CapturedState, HandlerError>;
}

fn capture(
    representation: CapturedRepresentation,
    documented_default: Option<bool>,
    externally_managed: bool,
) -> Result<CapturedState, HandlerError> {
    let (effective_enabled, effective_state_known) = match representation {
        CapturedRepresentation::Missing => (
            documented_default.unwrap_or(false),
            documented_default.is_some(),
        ),
        CapturedRepresentation::Dword(0) => (false, true),
        CapturedRepresentation::Dword(1) => (true, true),
        CapturedRepresentation::Dword(_) => {
            return Err(HandlerError::new(
                HandlerErrorKind::InvalidRepresentation,
                "The fixed registry value contained an unsupported DWORD.",
            ));
        }
    };
    Ok(CapturedState {
        representation,
        effective_enabled,
        effective_state_known,
        authority: if externally_managed {
            "external_policy".into()
        } else {
            "user".into()
        },
        confidence: "confirmed_representation".into(),
        captured_at: crate::inspection::timestamp(),
    })
}

pub(super) fn expected(target: MutationTarget) -> CapturedRepresentation {
    CapturedRepresentation::Dword(u32::from(target.enabled()))
}

fn verify_target(
    state: CapturedState,
    target: MutationTarget,
) -> Result<CapturedState, HandlerError> {
    require_user_authority(&state)?;
    if state.effective_state_known
        && state.representation == expected(target)
        && state.effective_enabled == target.enabled()
    {
        Ok(state)
    } else {
        Err(HandlerError::new(
            HandlerErrorKind::VerificationMismatch,
            "The effective setting did not match the reviewed target.",
        ))
    }
}

fn verify_exact(
    state: CapturedState,
    expected: &CapturedState,
) -> Result<CapturedState, HandlerError> {
    require_user_authority(&state)?;
    if state.representation == expected.representation
        && state.effective_state_known == expected.effective_state_known
        && state.effective_enabled == expected.effective_enabled
    {
        Ok(state)
    } else {
        Err(HandlerError::new(
            HandlerErrorKind::VerificationMismatch,
            "Rollback did not restore the exact captured representation.",
        ))
    }
}

fn require_user_authority(state: &CapturedState) -> Result<(), HandlerError> {
    if state.authority == "user" {
        Ok(())
    } else {
        Err(HandlerError::new(
            HandlerErrorKind::PolicyOverride,
            "A fixed Windows policy representation controls or blocks this taskbar setting.",
        ))
    }
}
