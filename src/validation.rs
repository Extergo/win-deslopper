//! Developer-only deterministic VM fixture validation. This module consumes the
//! same parsers and detectors as production but cannot execute or mutate Windows.

use crate::inspection::{
    CancellationToken, CommandResult, QueryFailure, QueryId, ReadOnlyRunner, run_inspection,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureBundle {
    pub fixture_schema_version: u32,
    pub scenario_id: String,
    pub os_edition: String,
    pub os_build: u32,
    pub queries: BTreeMap<QueryId, Value>,
    #[serde(default)]
    pub query_failures: BTreeMap<QueryId, FixtureFailure>,
    pub expected: Vec<FixtureAssertion>,
    pub known_limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureFailure {
    pub kind: crate::inspection::QueryErrorKind,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureAssertion {
    pub component_id: String,
    pub detector_status: String,
    pub applicability_status: String,
    pub package_completeness: Option<String>,
    pub authority: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureValidationResult {
    pub scenario_id: String,
    pub passed: bool,
    pub assertions_checked: usize,
    pub failures: Vec<String>,
}

struct FixtureRunner<'a>(&'a FixtureBundle);

impl ReadOnlyRunner for FixtureRunner<'_> {
    fn run(&self, query_id: QueryId, _: &CancellationToken) -> Result<CommandResult, QueryFailure> {
        if let Some(error) = self.0.query_failures.get(&query_id) {
            return Err(QueryFailure {
                query_id,
                kind: error.kind,
                message: error.message.clone(),
                exit_code: None,
                duration_ms: 0,
                partial_output_available: false,
            });
        }
        let value = self
            .0
            .queries
            .get(&query_id)
            .cloned()
            .unwrap_or(Value::Null);
        Ok(CommandResult {
            query_id,
            exit_code: Some(0),
            stdout: value.to_string(),
            duration_ms: 0,
        })
    }
}

pub fn validate_fixture(bundle: &FixtureBundle) -> FixtureValidationResult {
    let token = CancellationToken::default();
    let outcome = run_inspection(
        &FixtureRunner(bundle),
        format!("fixture-{}", bundle.scenario_id),
        &token,
        |_| {},
    );
    let mut failures = Vec::new();
    if bundle.fixture_schema_version != 1 {
        failures.push(format!(
            "Unsupported fixture schema {}",
            bundle.fixture_schema_version
        ));
    }
    if bundle.known_limitations.is_empty() {
        failures.push(
            "Fixture must document known limitations, or explicitly state that none are known"
                .into(),
        );
    }
    if outcome.platform.build != bundle.os_build {
        failures.push(format!(
            "Expected build {}, detected {}",
            bundle.os_build, outcome.platform.build
        ));
    }
    if !outcome
        .platform
        .edition
        .eq_ignore_ascii_case(&bundle.os_edition)
    {
        failures.push(format!(
            "Expected edition {}, detected {}",
            bundle.os_edition, outcome.platform.edition
        ));
    }
    for expected in &bundle.expected {
        let Some(observation) = outcome
            .observations
            .iter()
            .find(|observation| observation.component_id.key() == expected.component_id)
        else {
            failures.push(format!("{} produced no observation", expected.component_id));
            continue;
        };
        let detector = format!("{:?}", observation.detector_status).to_ascii_lowercase();
        if detector
            != expected
                .detector_status
                .replace('_', "")
                .to_ascii_lowercase()
            && detector != expected.detector_status.to_ascii_lowercase()
        {
            failures.push(format!(
                "{} detector status: expected {}, got {:?}",
                expected.component_id, expected.detector_status, observation.detector_status
            ));
        }
        let applicability = format!("{:?}", observation.applicability.status).to_ascii_lowercase();
        if applicability
            != expected
                .applicability_status
                .replace('_', "")
                .to_ascii_lowercase()
        {
            failures.push(format!(
                "{} applicability: expected {}, got {:?}",
                expected.component_id,
                expected.applicability_status,
                observation.applicability.status
            ));
        }
        if let Some(completeness) = &expected.package_completeness
            && !format!("{:?}", observation.package_completeness)
                .eq_ignore_ascii_case(&completeness.replace('_', ""))
        {
            failures.push(format!(
                "{} package completeness: expected {}, got {:?}",
                expected.component_id, completeness, observation.package_completeness
            ));
        }
        if let Some(authority) = &expected.authority
            && !format!("{:?}", observation.authority)
                .eq_ignore_ascii_case(&authority.replace('_', ""))
        {
            failures.push(format!(
                "{} authority: expected {}, got {:?}",
                expected.component_id, authority, observation.authority
            ));
        }
    }
    FixtureValidationResult {
        scenario_id: bundle.scenario_id.clone(),
        passed: failures.is_empty(),
        assertions_checked: bundle.expected.len(),
        failures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_checked_in_vm_fixture_matches_current_detectors() {
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("validation/fixtures");
        let mut checked = 0;
        for entry in std::fs::read_dir(directory).expect("fixture directory must exist") {
            let path = entry.expect("fixture entry must be readable").path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path).expect("fixture must be readable");
            let bundle: FixtureBundle =
                serde_json::from_str(&raw).expect("fixture schema must be valid");
            let report = validate_fixture(&bundle);
            assert!(
                report.passed,
                "{} failed: {}",
                report.scenario_id,
                report.failures.join("; ")
            );
            checked += 1;
        }
        assert!(checked > 0, "at least one captured VM fixture is required");
    }
}
