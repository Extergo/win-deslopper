//! Search Highlights was not given a write handler because the documented
//! control is device policy. The approved current-user substitute is the
//! documented Show Desktop corner taskbar setting.

use super::{
    HandlerError, MutationBackend, OperationHandler, capture, expected, verify_exact, verify_target,
};
use crate::mutation::{
    plan::{CapturedRepresentation, CapturedState},
    request::{MutationOperationId, MutationTarget},
};

pub struct TaskbarShowDesktopHandler;

impl OperationHandler for TaskbarShowDesktopHandler {
    fn operation_id(&self) -> MutationOperationId {
        MutationOperationId::ShowDesktopEnabled
    }

    fn inspect_pre_state(
        &self,
        backend: &dyn MutationBackend,
    ) -> Result<CapturedState, HandlerError> {
        capture(
            backend.read_show_desktop()?,
            Some(true),
            backend.show_desktop_externally_managed()?,
        )
    }

    fn validate_target(&self, target: MutationTarget) -> Result<(), HandlerError> {
        let _ = target;
        Ok(())
    }

    fn apply(
        &self,
        backend: &dyn MutationBackend,
        target: MutationTarget,
    ) -> Result<(), HandlerError> {
        backend.write_show_desktop(&expected(target))
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
        backend.write_show_desktop(&self.build_rollback(pre_state))
    }

    fn verify_rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<CapturedState, HandlerError> {
        verify_exact(self.inspect_pre_state(backend)?, pre_state)
    }
}
