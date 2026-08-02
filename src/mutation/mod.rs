//! Internal-only, feature-gated mutation broker.

pub mod broker;
pub mod handlers;
pub mod journal;
pub mod live_validation;
pub mod plan;
pub mod request;
pub mod rollback;
pub mod transaction;
pub mod verification;

pub use broker::{Broker, BrokerContext, BrokerError};
pub use journal::MutationJournal;
pub use live_validation::{EvidenceExportRequest, LiveEvidenceBundle, build_live_validation_gate};
pub use request::{
    AlphaGateStatus, ApprovalRequest, MutationOperationId, PlanRequest, RollbackRequest,
};
pub use transaction::MutationTransaction;

pub const HANDLER_VERSION: &str = "mutation-alpha.1";
