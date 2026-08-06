use super::{
    HandlerError, MutationBackend, OperationHandler, capture, expected, verify_exact, verify_target,
};
use crate::mutation::{
    plan::{CapturedRepresentation, CapturedState},
    request::{MutationOperationId, MutationTarget},
};

pub struct TaskbarWidgetsHandler;

impl OperationHandler for TaskbarWidgetsHandler {
    fn operation_id(&self) -> MutationOperationId {
        MutationOperationId::WidgetsVisibility
    }

    fn inspect_pre_state(
        &self,
        backend: &dyn MutationBackend,
    ) -> Result<CapturedState, HandlerError> {
        capture(
            backend.read_widgets()?,
            None,
            backend.widgets_externally_managed()?,
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
        backend.write_widgets(&expected(target))
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
        backend.write_widgets(&self.build_rollback(pre_state))
    }

    fn verify_rollback(
        &self,
        backend: &dyn MutationBackend,
        pre_state: &CapturedState,
    ) -> Result<CapturedState, HandlerError> {
        verify_exact(self.inspect_pre_state(backend)?, pre_state)
    }
}
