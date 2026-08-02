use super::plan::CapturedState;

pub fn conflicts_with_applied_state(current: &CapturedState, applied: &CapturedState) -> bool {
    current.representation != applied.representation
        || current.effective_enabled != applied.effective_enabled
}
