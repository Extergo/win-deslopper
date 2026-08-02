use super::{handlers::HandlerError, plan::CapturedState, request::MutationTarget};

pub fn classify_apply(
    before: &CapturedState,
    after: &CapturedState,
    target: MutationTarget,
) -> Result<&'static str, HandlerError> {
    if after.effective_enabled != target.enabled() {
        return Ok("write_succeeded_effective_state_differs");
    }
    if before.effective_enabled == target.enabled() {
        Ok("already_compliant")
    } else {
        Ok("applied_and_verified")
    }
}
