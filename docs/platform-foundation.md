# Platform foundation

`platform.rs` is the pure domain contract for the exact 20 v1 components. `applicability.rs` owns versioned build/edition rules and `package_identity.rs` owns exact identity migrations. None executes Windows commands.

The contract separates effective state, user preference, policy layers, authority attribution and confidence, applicability, detector status, package registration/provisioning and completeness, OneDrive Files On-Demand composite state, structured evidence, and warnings/errors. Unknown, unsupported, failed, cancelled, permission-limited, and absent states cannot be substituted for one another.

Drift distinguishes package presence, provisioning, normal version servicing, identity migration, policy, preference, management authority, availability/applicability, detection uncertainty, and reset/reinstall signals. Failed and cancelled observations are excluded as proof of drift. Authority refinement and overridden preference changes may be historical information without becoming alarming effective-state drift.

Desired-state and preview APIs consume these Rust rules. TypeScript contains display types only and cannot construct arbitrary state variants or duplicate applicability and authority resolution.

The read-only inspection and desired-state preview contracts still have no generic executor. The separately compiled Mutation Broker Alpha exposes only three closed, fixed-key taskbar operations documented in `mutation-broker-alpha.md`; no other v1 component is mutation-capable, and preview records cannot be executed directly.
