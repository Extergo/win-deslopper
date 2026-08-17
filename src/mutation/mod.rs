//! Internal-only, feature-gated mutation broker.

pub mod broker;
#[cfg(feature = "mutation-alpha")]
pub mod governance;
pub mod handlers;
pub mod journal;
#[cfg(feature = "mutation-alpha")]
pub mod live_validation;
pub mod package;
pub mod plan;
mod process_lock;
pub mod request;
pub mod rollback;
pub mod transaction;
#[cfg(feature = "mutation-alpha")]
pub mod verification;

pub use broker::{Broker, BrokerContext, BrokerError};
#[cfg(feature = "mutation-alpha")]
pub use journal::MutationJournal;
#[cfg(feature = "mutation-alpha")]
pub use live_validation::{EvidenceExportRequest, LiveEvidenceBundle, build_live_validation_gate};
pub use package::{
    OwnerPackageRemoveRequest, OwnerPackageRestoreRequest, PackageActionability, PackageBroker,
    PackageDetectorEvidence, PackageMutationContext, PackageMutationTransaction,
    PackageOperationId, PackageOperationResult, WindowsPackageDeploymentBackend,
};
#[cfg(feature = "mutation-alpha")]
pub use request::{AlphaGateStatus, ApprovalRequest, PlanRequest, RollbackRequest};
pub use request::{MutationOperationId, OwnerApplyRequest, OwnerUndoRequest};
pub use transaction::MutationTransaction;

#[cfg(feature = "mutation-alpha")]
pub const HANDLER_VERSION: &str = "mutation-alpha.1";
pub const OWNER_HANDLER_VERSION_V1: &str = "owner-widgets.1";
pub const OWNER_HANDLER_VERSION_V2: &str = "owner-taskbar.2";
pub const OWNER_HANDLER_VERSION: &str = "owner-cleanup.3";

pub fn is_owner_handler_version(version: &str) -> bool {
    matches!(
        version,
        OWNER_HANDLER_VERSION_V1 | OWNER_HANDLER_VERSION_V2 | OWNER_HANDLER_VERSION
    )
}
