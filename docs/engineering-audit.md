# Engineering audit — 1 August 2026

The repository is a Tauri 2 desktop shell with a SvelteKit 5 frontend and a
Rust 2024 backend. The accepted alpha had seven package detectors using only
`Get-AppxPackage -AllUsers`; OneDrive and twelve policy/preference components
returned `Unknown`. Provisioning, current-user separation, cancellation,
deduplicated drift, observation history UI, and effective authority were absent.
JSON snapshots could contain structured evidence but not raw command output.
Desired-state validation checked only that a component ID existed, and preview
plans were not anchored to an observation.

Retained: the Tauri/Svelte boundary, Rust-owned orchestration, preview-only
safety policy, and existing presentation tests. Refactored foundation: the
research-backed typed platform catalogue, state, evidence, authority, and drift
contracts in `src/platform.rs`. Not added: undocumented registry tweaks,
blanket service/task disabling, destructive AppX removal, security changes, or
fake apply behavior.

The beta replaces JSON persistence with SQLite, adds current-user/all-user/
provisioned AppX inventories, a privacy-preserving OneDrive detector, policy and
preference inspection, normalized observation history, and evidence cards in
the UI. All PowerShell remains fixed and read-only. JSON is imported once and
backed up; malformed input is retained and recorded as a migration failure.
