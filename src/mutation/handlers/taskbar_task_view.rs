//! Fixed handler for the current user's Task View taskbar button.

use super::{
    HandlerError, MutationBackend, OperationHandler, capture, expected, require_user_authority,
    verify_exact, verify_target,
};
use crate::mutation::{
    plan::{CapturedRepresentation, CapturedState},
    request::{MutationOperationId, MutationTarget},
};

pub struct TaskbarTaskViewHandler;

impl OperationHandler for TaskbarTaskViewHandler {
    fn operation_id(&self) -> MutationOperationId {
        MutationOperationId::TaskViewVisibility
    }

    fn inspect_pre_state(
        &self,
        backend: &dyn MutationBackend,
    ) -> Result<CapturedState, HandlerError> {
        capture(
            backend.read_task_view()?,
            Some(true),
            backend.task_view_externally_managed()?,
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
        backend.write_task_view(&expected(target))
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
        backend.write_task_view(&self.build_rollback(pre_state))
    }

    fn verify_rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<CapturedState, HandlerError> {
        verify_exact(self.inspect_pre_state(backend)?, pre_state)
    }
}
