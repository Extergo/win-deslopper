use std::collections::HashSet;

const COMMITTED_POLICY: &str = include_str!("../../.deslopper/policy.toml");
const POLICY_SCHEMA_KEY: &str = "live_validation_policy_schema_version";
const EXPLICIT_TARGET_KEY: &str =
    "allow_live_mutation_only_in_explicitly_approved_disposable_target";
const ALLOWED_TYPES_KEY: &str = "allowed_live_validation_target_types";
const STRICT_PHYSICAL_KEY: &str = "physical_target_requires_strict_recovery_readiness";
const LEGACY_VM_ONLY_KEY: &str = "allow_live_mutation_only_in_explicitly_approved_disposable_vm";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PolicyTargetType {
    VirtualMachine,
    PhysicalLaptop,
}

impl PolicyTargetType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "virtual_machine" => Some(Self::VirtualMachine),
            "physical_laptop" => Some(Self::PhysicalLaptop),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveValidationPolicy {
    pub schema_version: u32,
    allowed_target_types: HashSet<PolicyTargetType>,
    pub strict_physical_recovery_required: bool,
}

impl LiveValidationPolicy {
    pub fn parse(source: &str) -> Result<Self, &'static str> {
        let mut schema_version = None;
        let mut explicit_target_only = None;
        let mut allowed_target_types = None;
        let mut strict_physical_recovery_required = None;

        for line in source.lines() {
            let line = line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            let Some((raw_key, raw_value)) = line.split_once('=') else {
                continue;
            };
            let key = raw_key.trim();
            let value = raw_value.trim();
            match key {
                LEGACY_VM_ONLY_KEY => {
                    return Err("The obsolete VM-only live-validation policy is prohibited.");
                }
                POLICY_SCHEMA_KEY => {
                    set_once(&mut schema_version, parse_u32(value)?)?;
                }
                EXPLICIT_TARGET_KEY => {
                    set_once(&mut explicit_target_only, parse_bool(value)?)?;
                }
                ALLOWED_TYPES_KEY => {
                    set_once(&mut allowed_target_types, parse_target_types(value)?)?;
                }
                STRICT_PHYSICAL_KEY => {
                    set_once(&mut strict_physical_recovery_required, parse_bool(value)?)?;
                }
                _ => {}
            }
        }

        if schema_version != Some(2) {
            return Err("Live-validation policy schema version 2 is required.");
        }
        if explicit_target_only != Some(true) {
            return Err("Explicit approved disposable-target enforcement must be enabled.");
        }
        if strict_physical_recovery_required != Some(true) {
            return Err("Strict physical-target recovery enforcement must be enabled.");
        }
        let allowed_target_types = allowed_target_types
            .filter(|values| !values.is_empty())
            .ok_or("At least one explicit live-validation target type is required.")?;

        Ok(Self {
            schema_version: 2,
            allowed_target_types,
            strict_physical_recovery_required: true,
        })
    }

    pub fn allows(&self, target_type: PolicyTargetType) -> bool {
        self.allowed_target_types.contains(&target_type)
    }
}

pub fn committed_live_validation_policy() -> Result<LiveValidationPolicy, &'static str> {
    LiveValidationPolicy::parse(COMMITTED_POLICY)
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> Result<(), &'static str> {
    if slot.replace(value).is_some() {
        Err("A live-validation policy field is duplicated.")
    } else {
        Ok(())
    }
}

fn parse_bool(value: &str) -> Result<bool, &'static str> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("A live-validation policy Boolean is malformed."),
    }
}

fn parse_u32(value: &str) -> Result<u32, &'static str> {
    value
        .parse()
        .map_err(|_| "The live-validation policy version is malformed.")
}

fn parse_target_types(value: &str) -> Result<HashSet<PolicyTargetType>, &'static str> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or("The allowed target-type list is malformed.")?;
    let mut result = HashSet::new();
    if inner.trim().is_empty() {
        return Ok(result);
    }
    for raw in inner.split(',') {
        let name = raw
            .trim()
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or("Every allowed target type must be a quoted string.")?;
        let target_type = PolicyTargetType::parse(name)
            .ok_or("The policy contains an unknown live-validation target type.")?;
        if !result.insert(target_type) {
            return Err("The policy contains a duplicate live-validation target type.");
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
live_validation_policy_schema_version = 2
allow_live_mutation_only_in_explicitly_approved_disposable_target = true
allowed_live_validation_target_types = ["virtual_machine", "physical_laptop"]
physical_target_requires_strict_recovery_readiness = true
"#;

    #[test]
    fn explicit_disposable_target_policy_parses_and_allows_only_listed_types() {
        let policy = LiveValidationPolicy::parse(VALID).unwrap();
        assert!(policy.allows(PolicyTargetType::VirtualMachine));
        assert!(policy.allows(PolicyTargetType::PhysicalLaptop));

        let vm_only = VALID.replace(
            "[\"virtual_machine\", \"physical_laptop\"]",
            "[\"virtual_machine\"]",
        );
        let policy = LiveValidationPolicy::parse(&vm_only).unwrap();
        assert!(policy.allows(PolicyTargetType::VirtualMachine));
        assert!(!policy.allows(PolicyTargetType::PhysicalLaptop));
    }

    #[test]
    fn missing_malformed_unknown_and_legacy_policy_fail_closed() {
        assert!(LiveValidationPolicy::parse("").is_err());
        assert!(
            LiveValidationPolicy::parse(&VALID.replace("physical_laptop", "future_target"))
                .is_err()
        );
        assert!(
            LiveValidationPolicy::parse(
                "allow_live_mutation_only_in_explicitly_approved_disposable_vm = true"
            )
            .is_err()
        );
        assert!(LiveValidationPolicy::parse(&VALID.replace(" = true", " = yes")).is_err());
    }

    #[test]
    fn checked_in_policy_is_versioned_and_fail_closed() {
        let policy = committed_live_validation_policy().unwrap();
        assert_eq!(policy.schema_version, 2);
        assert!(policy.allows(PolicyTargetType::VirtualMachine));
        assert!(policy.allows(PolicyTargetType::PhysicalLaptop));
        assert!(policy.strict_physical_recovery_required);
    }
}
