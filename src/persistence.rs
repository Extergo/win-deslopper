//! Versioned SQLite persistence with one-time import of the alpha JSON store.

use crate::{
    inspection::{InspectionLifecycle, QueryFailure},
    platform::{ComponentId, DetectionResult, PlatformInfo, State},
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};

pub const SCHEMA_VERSION: u32 = 5;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Store {
    pub schema_version: u32,
    pub snapshots: Vec<Snapshot>,
    pub desired: Vec<DesiredState>,
    pub drift: Vec<DriftEvent>,
    pub desired_revisions: Vec<DesiredStateRevision>,
    pub preferences: ProductPreferences,
    pub database_status: DatabaseStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProductPreferences {
    pub history_retention_days: u32,
}

impl Default for ProductPreferences {
    fn default() -> Self {
        Self {
            history_retention_days: 180,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: String,
    pub platform: PlatformInfo,
    pub observations: Vec<DetectionResult>,
    pub lifecycle: Option<InspectionLifecycle>,
    pub query_failures: Vec<QueryFailure>,
    pub machine_id: String,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            id: String::new(),
            timestamp: String::new(),
            platform: crate::inspection::default_platform(),
            observations: Vec::new(),
            lifecycle: None,
            query_failures: Vec::new(),
            machine_id: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStatus {
    pub healthy: bool,
    pub message: String,
    pub original_path: String,
    pub recovery_available: bool,
    pub migration_status: String,
}

impl Default for DatabaseStatus {
    fn default() -> Self {
        Self {
            healthy: true,
            message: "SQLite database is healthy".into(),
            original_path: path().display().to_string(),
            recovery_available: false,
            migration_status: format!("schema_v{SCHEMA_VERSION}"),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DesiredState {
    pub component_id: ComponentId,
    pub state: State,
    pub scope: String,
    pub created_at: String,
    pub modified_at: String,
    pub persistent: bool,
    pub approval_required: bool,
    pub selected_build: u32,
    pub selected_edition: String,
    pub note: Option<String>,
    pub validation_status: String,
    pub revision: u32,
}

impl Default for DesiredState {
    fn default() -> Self {
        Self {
            component_id: ComponentId::Onedrive,
            state: State::Unknown {
                error: "No desired state".into(),
            },
            scope: String::new(),
            created_at: String::new(),
            modified_at: String::new(),
            persistent: false,
            approval_required: true,
            selected_build: 0,
            selected_edition: String::new(),
            note: None,
            validation_status: "requires_review".into(),
            revision: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesiredStateRevision {
    pub component_id: ComponentId,
    pub revision: u32,
    pub state: State,
    pub scope: String,
    pub changed_at: String,
    pub persistent: bool,
    pub approval_required: bool,
    pub note: Option<String>,
    pub validation_status: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DriftEvent {
    pub component_id: ComponentId,
    pub previous: State,
    pub current: State,
    pub desired: Option<State>,
    pub classification: String,
    pub cause: String,
    pub confidence: String,
    pub first_detected: String,
    pub last_observed: String,
    pub resolved: bool,
    pub reviewed: bool,
    pub reviewed_at: Option<String>,
    pub returned_to_desired: bool,
    pub occurrence_count: u32,
    pub supporting_facts: Vec<String>,
    pub alternative_causes: Vec<String>,
    pub inference_rule_version: u32,
    pub previous_inspection_id: Option<String>,
    pub current_inspection_id: Option<String>,
}

impl Default for DriftEvent {
    fn default() -> Self {
        Self {
            component_id: ComponentId::Onedrive,
            previous: State::Unknown {
                error: String::new(),
            },
            current: State::Unknown {
                error: String::new(),
            },
            desired: None,
            classification: String::new(),
            cause: String::new(),
            confidence: "Unknown".into(),
            first_detected: String::new(),
            last_observed: String::new(),
            resolved: false,
            reviewed: false,
            reviewed_at: None,
            returned_to_desired: false,
            occurrence_count: 1,
            supporting_facts: Vec::new(),
            alternative_causes: Vec::new(),
            inference_rule_version: 1,
            previous_inspection_id: None,
            current_inspection_id: None,
        }
    }
}

pub fn path() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(|r| PathBuf::from(r).join("Deslopper").join("deslopper.db"))
        .unwrap_or_else(|_| PathBuf::from(".deslopper/deslopper.db"))
}
fn json_path() -> PathBuf {
    path().with_file_name("store.json")
}

pub fn load() -> Store {
    match open() {
        Ok(conn) => load_store(&conn).unwrap_or_else(|error| unavailable_store(error.to_string())),
        Err(error) => unavailable_store(error.to_string()),
    }
}

fn unavailable_store(message: String) -> Store {
    Store {
        schema_version: SCHEMA_VERSION,
        database_status: DatabaseStatus {
            healthy: false,
            message: format!("The database was preserved but could not be opened: {message}"),
            original_path: path().display().to_string(),
            recovery_available: true,
            migration_status: "failed".into(),
        },
        ..Default::default()
    }
}
pub fn save(store: &Store) -> Result<(), String> {
    let mut conn = open().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    persist(&tx, store).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub fn apply_history_retention(store: &mut Store) -> Result<(), String> {
    retain_history(store, crate::inspection::timestamp().parse().unwrap_or(0));
    replace_read_only_data_at(store, &path())
}

fn retain_history(store: &mut Store, now: u128) {
    let days = store.preferences.history_retention_days;
    if days == 0 || store.snapshots.len() <= 1 {
        return;
    }
    let cutoff = now.saturating_sub(u128::from(days) * 86_400_000);
    let latest_id = store.snapshots.last().map(|snapshot| snapshot.id.clone());
    store.snapshots.retain(|snapshot| {
        Some(&snapshot.id) == latest_id.as_ref()
            || snapshot
                .timestamp
                .parse::<u128>()
                .is_ok_and(|timestamp| timestamp >= cutoff)
    });
    let retained_ids: Vec<&str> = store
        .snapshots
        .iter()
        .map(|snapshot| snapshot.id.as_str())
        .collect();
    store.drift.retain(|event| {
        event
            .current_inspection_id
            .as_deref()
            .is_none_or(|id| retained_ids.contains(&id))
    });
}

pub fn clear_local_history(store: &mut Store) -> Result<(), String> {
    store.snapshots.clear();
    store.desired.clear();
    store.desired_revisions.clear();
    store.drift.clear();
    replace_read_only_data_at(store, &path())
}

fn replace_read_only_data_at(store: &Store, target: &std::path::Path) -> Result<(), String> {
    let mut conn = open_at(target).map_err(|error| error.to_string())?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    tx.execute_batch(
        "DELETE FROM package_observations;
         DELETE FROM component_observations;
         DELETE FROM drift_events;
         DELETE FROM preview_plans;
         DELETE FROM desired_state_revisions;
         DELETE FROM desired_states;
         DELETE FROM inspections;
         DELETE FROM machines;",
    )
    .map_err(|error| error.to_string())?;
    persist(&tx, store).map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

pub fn save_preview_plan(component_id: &str, preview: &serde_json::Value) -> Result<(), String> {
    let conn = open().map_err(|e| e.to_string())?;
    let now = crate::inspection::timestamp();
    let id = format!("plan-{component_id}-{now}");
    conn.execute("INSERT INTO preview_plans(id,component_id,source_observation_id,generated_at,status,preview_json,execution_unavailable_reason) VALUES(?1,?2,(SELECT id FROM component_observations WHERE component_id=?2 ORDER BY id DESC LIMIT 1),?3,'preview_only',?4,'Automatic restoration is unavailable in Product Alpha')",params![id,component_id,now,preview.to_string()]).map_err(|e|e.to_string())?;
    Ok(())
}

pub(crate) fn open() -> rusqlite::Result<Connection> {
    let target = path();
    let conn = open_at(&target)?;
    import_json_once(&conn)?;
    let integrity: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(conn)
}

pub(crate) fn open_at(target: &std::path::Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?
    }
    let conn = Connection::open(target)?;
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE;
      CREATE TABLE IF NOT EXISTS database_metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS machines(id TEXT PRIMARY KEY,device_name_hash TEXT NOT NULL,first_observed TEXT NOT NULL,last_observed TEXT NOT NULL,reset_identity TEXT,current_edition TEXT,current_build INTEGER,architecture TEXT);
      CREATE TABLE IF NOT EXISTS inspections(id TEXT PRIMARY KEY,machine_id TEXT NOT NULL,started_at TEXT NOT NULL,completed_at TEXT,app_version TEXT NOT NULL,windows_build INTEGER,windows_edition TEXT,status TEXT NOT NULL,successful_count INTEGER NOT NULL,unknown_count INTEGER NOT NULL,failed_count INTEGER NOT NULL,cancelled INTEGER NOT NULL DEFAULT 0,platform_json TEXT NOT NULL,FOREIGN KEY(machine_id) REFERENCES machines(id));
      CREATE TABLE IF NOT EXISTS component_observations(id INTEGER PRIMARY KEY AUTOINCREMENT,inspection_id TEXT NOT NULL,component_id TEXT NOT NULL,current_state_json TEXT NOT NULL,user_preference_state TEXT,effective_state TEXT,policy_state TEXT,authority TEXT NOT NULL,applicable INTEGER NOT NULL,confidence INTEGER NOT NULL,warning_count INTEGER NOT NULL,error_count INTEGER NOT NULL,evidence_summary TEXT NOT NULL,evidence_json TEXT NOT NULL,observed_at TEXT NOT NULL,UNIQUE(inspection_id,component_id),FOREIGN KEY(inspection_id) REFERENCES inspections(id));
      CREATE TABLE IF NOT EXISTS package_observations(id INTEGER PRIMARY KEY AUTOINCREMENT,observation_id INTEGER NOT NULL,package_family_name TEXT,package_full_name TEXT,version TEXT,architecture TEXT,publisher TEXT,current_user_registered INTEGER,other_user_registered INTEGER,all_users_present INTEGER,provisioned INTEGER,framework INTEGER,resource_package INTEGER,dependency INTEGER,non_removable INTEGER,detection_completeness TEXT NOT NULL,FOREIGN KEY(observation_id) REFERENCES component_observations(id));
      CREATE TABLE IF NOT EXISTS desired_states(component_id TEXT PRIMARY KEY,requested_state_json TEXT NOT NULL,scope TEXT NOT NULL,created_at TEXT NOT NULL,modified_at TEXT NOT NULL,selected_build INTEGER NOT NULL,selected_edition TEXT NOT NULL,persistent_remediation INTEGER NOT NULL,always_require_approval INTEGER NOT NULL,user_note TEXT,validation_status TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS drift_events(id INTEGER PRIMARY KEY AUTOINCREMENT,component_id TEXT NOT NULL,previous_state_json TEXT NOT NULL,current_state_json TEXT NOT NULL,desired_state_json TEXT,classification TEXT NOT NULL,likely_cause TEXT NOT NULL,cause_confidence TEXT NOT NULL,first_detected TEXT NOT NULL,last_observed TEXT NOT NULL,occurrence_count INTEGER NOT NULL DEFAULT 1,resolved INTEGER NOT NULL DEFAULT 0,resolved_at TEXT,UNIQUE(component_id,classification,resolved));
      CREATE TABLE IF NOT EXISTS preview_plans(id TEXT PRIMARY KEY,component_id TEXT NOT NULL,source_observation_id INTEGER,generated_at TEXT NOT NULL,status TEXT NOT NULL,preview_json TEXT NOT NULL,execution_unavailable_reason TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS migration_log(name TEXT PRIMARY KEY,completed_at TEXT NOT NULL,status TEXT NOT NULL,detail TEXT);
      COMMIT;")?;
    let existing_version: u32 = conn
        .query_row(
            "SELECT value FROM database_metadata WHERE key='schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    if existing_version > SCHEMA_VERSION {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if existing_version < 2 {
        migrate_v2(conn)?;
    }
    if existing_version < 3 {
        migrate_v3(conn)?;
    }
    if existing_version < 4 {
        migrate_v4(conn)?;
    }
    if existing_version < 5 {
        migrate_v5(conn)?;
    }
    let now = crate::inspection::timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO database_metadata(key,value) VALUES('schema_version',?1)",
        [SCHEMA_VERSION.to_string()],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO database_metadata(key,value) VALUES('created_at',?1)",
        [&now],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO database_metadata(key,value) VALUES('last_migration_at',?1)",
        [&now],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO database_metadata(key,value) VALUES('application_version',?1)",
        [env!("CARGO_PKG_VERSION")],
    )?;
    Ok(())
}

fn migrate_v5(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys=OFF; BEGIN IMMEDIATE;")?;
    let result = conn.execute_batch(
        "CREATE TABLE mutation_transactions_v5(
           id TEXT PRIMARY KEY,
           plan_id TEXT NOT NULL UNIQUE,
           machine_id TEXT NOT NULL,
           component_id TEXT NOT NULL,
           operation_id TEXT NOT NULL,
           source_inspection_id TEXT NOT NULL,
           source_observation_id TEXT NOT NULL,
           desired_state_revision_id INTEGER,
           created_at TEXT NOT NULL,
           approved_at TEXT,
           started_at TEXT,
           completed_at TEXT,
           status TEXT NOT NULL CHECK(status IN (
             'created','validating','awaiting_approval','approved','capturing_pre_state',
             'applying','verifying','applied','no_change_needed','verification_failed','rollback_available',
             'rolling_back','rolled_back','rollback_verification_failed',
             'failed_before_mutation','failed_after_mutation','recovery_required',
             'cancelled_before_mutation')),
           required_privilege TEXT NOT NULL,
           handler_version TEXT NOT NULL,
           application_version TEXT NOT NULL,
           windows_build INTEGER NOT NULL,
           windows_edition TEXT NOT NULL,
           plan_hash TEXT NOT NULL,
           pre_state_hash TEXT,
           post_state_hash TEXT,
           rollback_state_hash TEXT,
           verification_result TEXT,
           error_category TEXT,
           error_summary TEXT,
           recovery_requirement TEXT,
           transaction_json TEXT NOT NULL,
           FOREIGN KEY(plan_id) REFERENCES mutation_plans(id)
         );
         INSERT INTO mutation_transactions_v5
         SELECT * FROM mutation_transactions;
         DROP TABLE mutation_transactions;
         ALTER TABLE mutation_transactions_v5 RENAME TO mutation_transactions;
         CREATE INDEX IF NOT EXISTS idx_mutation_history ON mutation_transactions(created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_mutation_recovery ON mutation_transactions(status,started_at);
         COMMIT;",
    );
    if result.is_err() {
        let _ = conn.execute_batch("ROLLBACK;");
    }
    let foreign_keys = conn.execute_batch("PRAGMA foreign_keys=ON;");
    result.and(foreign_keys)
}

fn migrate_v4(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    let result = (|| {
        add_column(conn, "drift_events", "reviewed INTEGER NOT NULL DEFAULT 0")?;
        add_column(conn, "drift_events", "reviewed_at TEXT")?;
        add_column(
            conn,
            "drift_events",
            "returned_to_desired INTEGER NOT NULL DEFAULT 0",
        )?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS product_preferences(
               key TEXT PRIMARY KEY,
               value TEXT NOT NULL
             );
             INSERT OR IGNORE INTO product_preferences(key,value)
             VALUES('history_retention_days','180');",
        )?;
        Ok(())
    })();
    match result {
        Ok(()) => conn.execute_batch("COMMIT;"),
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn migrate_v3(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "BEGIN IMMEDIATE;
         CREATE TABLE IF NOT EXISTS mutation_plans(
           id TEXT PRIMARY KEY,
           machine_id TEXT NOT NULL,
           source_inspection_id TEXT NOT NULL,
           source_observation_id TEXT NOT NULL,
           component_id TEXT NOT NULL,
           operation_id TEXT NOT NULL,
           generated_at TEXT NOT NULL,
           expires_at TEXT NOT NULL,
           plan_hash TEXT NOT NULL,
           approval_nonce_hash TEXT NOT NULL,
           consumed_at TEXT,
           plan_json TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS mutation_transactions(
           id TEXT PRIMARY KEY,
           plan_id TEXT NOT NULL UNIQUE,
           machine_id TEXT NOT NULL,
           component_id TEXT NOT NULL,
           operation_id TEXT NOT NULL,
           source_inspection_id TEXT NOT NULL,
           source_observation_id TEXT NOT NULL,
           desired_state_revision_id INTEGER,
           created_at TEXT NOT NULL,
           approved_at TEXT,
           started_at TEXT,
           completed_at TEXT,
           status TEXT NOT NULL CHECK(status IN (
             'created','validating','awaiting_approval','approved','capturing_pre_state',
             'applying','verifying','applied','no_change_needed','verification_failed','rollback_available',
             'rolling_back','rolled_back','rollback_verification_failed',
             'failed_before_mutation','failed_after_mutation','recovery_required',
             'cancelled_before_mutation')),
           required_privilege TEXT NOT NULL,
           handler_version TEXT NOT NULL,
           application_version TEXT NOT NULL,
           windows_build INTEGER NOT NULL,
           windows_edition TEXT NOT NULL,
           plan_hash TEXT NOT NULL,
           pre_state_hash TEXT,
           post_state_hash TEXT,
           rollback_state_hash TEXT,
           verification_result TEXT,
           error_category TEXT,
           error_summary TEXT,
           recovery_requirement TEXT,
           transaction_json TEXT NOT NULL,
           FOREIGN KEY(plan_id) REFERENCES mutation_plans(id)
         );
         CREATE TABLE IF NOT EXISTS mutation_steps(
           transaction_id TEXT NOT NULL,
           sequence_number INTEGER NOT NULL,
           step_type TEXT NOT NULL,
           started_at TEXT NOT NULL,
           completed_at TEXT,
           status TEXT NOT NULL,
           redacted_evidence_json TEXT NOT NULL,
           error_category TEXT,
           error_summary TEXT,
           PRIMARY KEY(transaction_id,sequence_number),
           FOREIGN KEY(transaction_id) REFERENCES mutation_transactions(id)
         );
         CREATE TABLE IF NOT EXISTS mutation_state_captures(
           transaction_id TEXT NOT NULL,
           capture_type TEXT NOT NULL CHECK(capture_type IN ('pre_state','post_state','rollback_state')),
           state_json TEXT NOT NULL,
           evidence_summary_json TEXT NOT NULL,
           captured_at TEXT NOT NULL,
           integrity_hash TEXT NOT NULL,
           PRIMARY KEY(transaction_id,capture_type),
           FOREIGN KEY(transaction_id) REFERENCES mutation_transactions(id)
         );
         CREATE TABLE IF NOT EXISTS mutation_rollbacks(
           transaction_id TEXT PRIMARY KEY,
           rollback_plan_json TEXT NOT NULL,
           rollback_available INTEGER NOT NULL,
           rollback_complete INTEGER NOT NULL,
           attempted_at TEXT,
           result TEXT,
           verification_result TEXT,
           conflict_detected INTEGER NOT NULL DEFAULT 0,
           FOREIGN KEY(transaction_id) REFERENCES mutation_transactions(id)
         );
         CREATE INDEX IF NOT EXISTS idx_mutation_history ON mutation_transactions(created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_mutation_recovery ON mutation_transactions(status,started_at);
         COMMIT;",
    )
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn add_column(conn: &Connection, table: &str, definition: &str) -> rusqlite::Result<()> {
    let column = definition.split_whitespace().next().unwrap_or_default();
    if !column_exists(conn, table, column)? {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {definition};"))?;
    }
    Ok(())
}

fn migrate_v2(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    let result = (|| {
        for definition in [
            "phase TEXT",
            "lifecycle_json TEXT",
            "total_detector_count INTEGER NOT NULL DEFAULT 0",
            "completed_detector_count INTEGER NOT NULL DEFAULT 0",
            "cancelled_count INTEGER NOT NULL DEFAULT 0",
            "warning_count INTEGER NOT NULL DEFAULT 0",
            "error_count INTEGER NOT NULL DEFAULT 0",
            "query_failures_json TEXT NOT NULL DEFAULT '[]'",
            "duration_ms INTEGER",
            "drift_count INTEGER NOT NULL DEFAULT 0",
            "management_summary TEXT",
        ] {
            add_column(conn, "inspections", definition)?;
        }
        for definition in [
            "applicability_json TEXT NOT NULL DEFAULT '{}'",
            "authority_attribution_json TEXT NOT NULL DEFAULT '{}'",
            "control_precedence_json TEXT NOT NULL DEFAULT '{}'",
            "detector_status TEXT NOT NULL DEFAULT 'unknown'",
            "package_completeness TEXT NOT NULL DEFAULT 'unknown'",
            "package_identities_json TEXT NOT NULL DEFAULT '[]'",
            "warning_json TEXT NOT NULL DEFAULT '[]'",
            "error_json TEXT",
        ] {
            add_column(conn, "component_observations", definition)?;
        }
        for definition in [
            "inspection_id TEXT",
            "component_id TEXT",
            "package_name TEXT",
            "bundle INTEGER NOT NULL DEFAULT 0",
            "install_location_present INTEGER",
            "dependencies_json TEXT NOT NULL DEFAULT '[]'",
            "permission_status TEXT NOT NULL DEFAULT 'unknown'",
            "observed_at TEXT",
            "current_user_state TEXT NOT NULL DEFAULT 'unknown'",
            "other_user_state TEXT NOT NULL DEFAULT 'unknown'",
            "provisioning_state TEXT NOT NULL DEFAULT 'unknown'",
        ] {
            add_column(conn, "package_observations", definition)?;
        }
        add_column(
            conn,
            "desired_states",
            "revision INTEGER NOT NULL DEFAULT 1",
        )?;
        for definition in [
            "supporting_facts_json TEXT NOT NULL DEFAULT '[]'",
            "alternative_causes_json TEXT NOT NULL DEFAULT '[]'",
            "inference_rule_version INTEGER NOT NULL DEFAULT 1",
            "previous_inspection_id TEXT",
            "current_inspection_id TEXT",
        ] {
            add_column(conn, "drift_events", definition)?;
        }
        conn.execute_batch("CREATE TABLE IF NOT EXISTS desired_state_revisions(id INTEGER PRIMARY KEY AUTOINCREMENT,component_id TEXT NOT NULL,revision INTEGER NOT NULL,requested_state_json TEXT NOT NULL,scope TEXT NOT NULL,changed_at TEXT NOT NULL,persistent_remediation INTEGER NOT NULL,always_require_approval INTEGER NOT NULL,user_note TEXT,validation_status TEXT NOT NULL,UNIQUE(component_id,revision));
          CREATE INDEX IF NOT EXISTS idx_inspections_machine_completed ON inspections(machine_id,completed_at DESC);
          CREATE INDEX IF NOT EXISTS idx_component_timeline ON component_observations(component_id,observed_at DESC);
          CREATE INDEX IF NOT EXISTS idx_package_inspection_component ON package_observations(inspection_id,component_id);
          CREATE UNIQUE INDEX IF NOT EXISTS ux_package_observation_identity ON package_observations(observation_id,package_name,package_full_name);
          CREATE INDEX IF NOT EXISTS idx_drift_history ON drift_events(resolved,classification,last_observed DESC);")?;
        Ok(())
    })();
    match result {
        Ok(()) => conn.execute_batch("COMMIT;"),
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn import_json_once(conn: &Connection) -> rusqlite::Result<()> {
    import_json_from(conn, &json_path())
}

fn parse_alpha_store(raw: &str) -> Result<Store, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    if let Ok(store) = serde_json::from_value::<Store>(value.clone()) {
        return Ok(store);
    }
    let mut store = Store {
        schema_version: value
            .get("schema_version")
            .or_else(|| value.get("schemaVersion"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1) as u32,
        ..Default::default()
    };
    if let Some(items) = value.get("snapshots").and_then(serde_json::Value::as_array) {
        store.snapshots = items
            .iter()
            .filter_map(|item| serde_json::from_value::<Snapshot>(item.clone()).ok())
            .filter(|snapshot| !snapshot.id.is_empty() && !snapshot.timestamp.is_empty())
            .collect();
    }
    if let Some(items) = value.get("desired").and_then(serde_json::Value::as_array) {
        store.desired = items
            .iter()
            .filter_map(|item| serde_json::from_value::<DesiredState>(item.clone()).ok())
            .filter(|desired| !desired.scope.is_empty())
            .collect();
    }
    if let Some(items) = value.get("drift").and_then(serde_json::Value::as_array) {
        store.drift = items
            .iter()
            .filter_map(|item| serde_json::from_value::<DriftEvent>(item.clone()).ok())
            .filter(|event| !event.classification.is_empty())
            .collect();
    }
    Ok(store)
}

fn import_json_from(conn: &Connection, source: &std::path::Path) -> rusqlite::Result<()> {
    let done: Option<String> = conn
        .query_row(
            "SELECT status FROM migration_log WHERE name='alpha_json'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if done
        .as_deref()
        .is_some_and(|status| status.starts_with("imported") || status == "not_found")
    {
        return Ok(());
    }
    if !source.exists() {
        conn.execute("INSERT INTO migration_log(name,completed_at,status,detail) VALUES('alpha_json',?1,'not_found',NULL)",[crate::inspection::timestamp()])?;
        return Ok(());
    }
    let raw = match fs::read_to_string(source) {
        Ok(v) => v,
        Err(e) => {
            conn.execute("INSERT INTO migration_log(name,completed_at,status,detail) VALUES('alpha_json',?1,'failed',?2) ON CONFLICT(name) DO UPDATE SET completed_at=excluded.completed_at,status=excluded.status,detail=excluded.detail",params![crate::inspection::timestamp(),e.to_string()])?;
            return Ok(());
        }
    };
    let store: Store = match parse_alpha_store(&raw) {
        Ok(v) => v,
        Err(e) => {
            conn.execute("INSERT INTO migration_log(name,completed_at,status,detail) VALUES('alpha_json',?1,'failed',?2) ON CONFLICT(name) DO UPDATE SET completed_at=excluded.completed_at,status=excluded.status,detail=excluded.detail",params![crate::inspection::timestamp(),e.to_string()])?;
            return Ok(());
        }
    };
    let tx = conn.unchecked_transaction()?;
    persist(&tx, &store)?;
    tx.commit()?;
    let mut backup = source.with_extension("json.migrated.bak");
    let mut collision = 1_u32;
    while backup.exists() {
        backup = source.with_extension(format!("json.migrated.{collision}.bak"));
        collision += 1;
    }
    let status = if fs::rename(source, &backup).is_ok() {
        "imported"
    } else {
        "imported_backup_failed"
    };
    conn.execute("INSERT INTO migration_log(name,completed_at,status,detail) VALUES('alpha_json',?1,?2,NULL) ON CONFLICT(name) DO UPDATE SET completed_at=excluded.completed_at,status=excluded.status,detail=NULL",params![crate::inspection::timestamp(),status])?;
    Ok(())
}

fn persist(tx: &Transaction<'_>, store: &Store) -> rusqlite::Result<()> {
    for snapshot in &store.snapshots {
        let machine_id = machine_id(&snapshot.platform);
        let device_hash = hash(
            snapshot
                .platform
                .device_name
                .as_deref()
                .unwrap_or("unknown"),
        );
        tx.execute("INSERT INTO machines(id,device_name_hash,first_observed,last_observed,current_edition,current_build,architecture) VALUES(?1,?2,?3,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET last_observed=excluded.last_observed,current_edition=excluded.current_edition,current_build=excluded.current_build,architecture=excluded.architecture",params![machine_id,device_hash,snapshot.timestamp,snapshot.platform.edition,snapshot.platform.build,snapshot.platform.architecture])?;
        let unknown = snapshot
            .observations
            .iter()
            .filter(|o| matches!(o.current, State::Unknown { .. }))
            .count() as i64;
        let failed = snapshot
            .observations
            .iter()
            .filter(|o| o.error.is_some())
            .count() as i64;
        let lifecycle = snapshot.lifecycle.as_ref();
        let mut redacted_platform = snapshot.platform.clone();
        redacted_platform.device_name = None;
        redacted_platform.user_sid = None;
        redacted_platform.owner_scope_id = None;
        let phase = lifecycle
            .map(|value| format!("{:?}", value.phase))
            .unwrap_or_else(|| "CompletedWithPartialFailures".into());
        let completed_at = lifecycle
            .and_then(|value| value.completed_at.clone())
            .unwrap_or_else(|| snapshot.timestamp.clone());
        let status = phase.to_ascii_lowercase();
        let successful = lifecycle
            .map(|value| value.successful_detector_count as i64)
            .unwrap_or((snapshot.observations.len() as i64) - unknown);
        let unknown = lifecycle
            .map(|value| value.unknown_detector_count as i64)
            .unwrap_or(unknown);
        let failed = lifecycle
            .map(|value| value.failed_detector_count as i64)
            .unwrap_or(failed);
        let cancelled_count = lifecycle
            .map(|value| value.cancelled_detector_count as i64)
            .unwrap_or(0);
        let duration_ms = completed_at
            .parse::<u128>()
            .ok()
            .zip(snapshot.timestamp.parse::<u128>().ok())
            .map(|(end, start)| end.saturating_sub(start) as i64);
        tx.execute("INSERT INTO inspections(id,machine_id,started_at,completed_at,app_version,windows_build,windows_edition,status,successful_count,unknown_count,failed_count,cancelled,platform_json,phase,lifecycle_json,total_detector_count,completed_detector_count,cancelled_count,warning_count,error_count,query_failures_json,duration_ms,management_summary) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23) ON CONFLICT(id) DO UPDATE SET completed_at=excluded.completed_at,status=excluded.status,successful_count=excluded.successful_count,unknown_count=excluded.unknown_count,failed_count=excluded.failed_count,cancelled=excluded.cancelled,platform_json=excluded.platform_json,phase=excluded.phase,lifecycle_json=excluded.lifecycle_json,total_detector_count=excluded.total_detector_count,completed_detector_count=excluded.completed_detector_count,cancelled_count=excluded.cancelled_count,warning_count=excluded.warning_count,error_count=excluded.error_count,query_failures_json=excluded.query_failures_json,duration_ms=excluded.duration_ms,management_summary=excluded.management_summary",params![snapshot.id,machine_id,snapshot.timestamp,completed_at,env!("CARGO_PKG_VERSION"),snapshot.platform.build,snapshot.platform.edition,status,successful,unknown,failed,(cancelled_count > 0) as i64,json(&redacted_platform)?,phase,lifecycle.map(json).transpose()?,lifecycle.map(|value|value.total_detector_count as i64).unwrap_or(snapshot.observations.len() as i64),lifecycle.map(|value|value.completed_detector_count as i64).unwrap_or(snapshot.observations.len() as i64),cancelled_count,lifecycle.map(|value|value.warning_count as i64).unwrap_or(0),lifecycle.map(|value|value.error_count as i64).unwrap_or(failed),json(&snapshot.query_failures)?,duration_ms,management_summary(&snapshot.platform)])?;
        for observation in &snapshot.observations {
            let confidence = observation
                .evidence
                .iter()
                .map(|e| e.confidence)
                .max()
                .unwrap_or(0);
            let summary = crate::privacy::redact(
                &observation
                    .evidence
                    .iter()
                    .map(|e| e.detail.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
            );
            let redacted_evidence: Vec<crate::platform::Evidence> = observation
                .evidence
                .iter()
                .cloned()
                .map(|mut item| {
                    item.detail = crate::privacy::redact(&item.detail);
                    item
                })
                .collect();
            tx.execute("INSERT INTO component_observations(inspection_id,component_id,current_state_json,user_preference_state,effective_state,policy_state,authority,applicable,confidence,warning_count,error_count,evidence_summary,evidence_json,observed_at,applicability_json,authority_attribution_json,control_precedence_json,detector_status,package_completeness,package_identities_json,warning_json,error_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22) ON CONFLICT(inspection_id,component_id) DO UPDATE SET current_state_json=excluded.current_state_json,user_preference_state=excluded.user_preference_state,effective_state=excluded.effective_state,policy_state=excluded.policy_state,authority=excluded.authority,applicable=excluded.applicable,confidence=excluded.confidence,warning_count=excluded.warning_count,error_count=excluded.error_count,evidence_summary=excluded.evidence_summary,evidence_json=excluded.evidence_json,observed_at=excluded.observed_at,applicability_json=excluded.applicability_json,authority_attribution_json=excluded.authority_attribution_json,control_precedence_json=excluded.control_precedence_json,detector_status=excluded.detector_status,package_completeness=excluded.package_completeness,package_identities_json=excluded.package_identities_json,warning_json=excluded.warning_json,error_json=excluded.error_json",params![snapshot.id,observation.component_id.key(),json(&observation.current)?,observation.preference_state,json(&observation.current)?,observation.policy_state,format!("{:?}",observation.authority),observation.applicable as i64,confidence,observation.warnings.len() as i64,observation.error.is_some() as i64,summary,json(&redacted_evidence)?,observation.detected_at,json(&observation.applicability)?,json(&observation.authority_attribution)?,json(&observation.control_precedence)?,format!("{:?}",observation.detector_status),format!("{:?}",observation.package_completeness),json(&observation.package_identities)?,json(&observation.warnings)?,observation.error])?;
            let observation_id: i64 = tx.query_row(
                "SELECT id FROM component_observations WHERE inspection_id=?1 AND component_id=?2",
                params![snapshot.id, observation.component_id.key()],
                |row| row.get(0),
            )?;
            tx.execute(
                "DELETE FROM package_observations WHERE observation_id=?1",
                [observation_id],
            )?;
            for package in &observation.packages {
                tx.execute("INSERT INTO package_observations(observation_id,inspection_id,component_id,package_family_name,package_full_name,package_name,version,architecture,publisher,current_user_registered,other_user_registered,all_users_present,provisioned,framework,resource_package,dependency,non_removable,detection_completeness,bundle,install_location_present,dependencies_json,permission_status,observed_at,current_user_state,other_user_state,provisioning_state) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,0,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25)",params![observation_id,snapshot.id,package.component_id.key(),package.package_family_name,package.package_full_name,package.package_name,package.version,package.architecture,package.publisher_id,(package.current_user==crate::platform::PackageRegistrationState::Present) as i64,(package.other_users==crate::platform::PackageRegistrationState::Present) as i64,(package.other_users==crate::platform::PackageRegistrationState::Present) as i64,(package.provisioning==crate::platform::PackageProvisioningState::Provisioned) as i64,package.framework as i64,package.resource_package as i64,package.non_removable as i64,format!("{:?}",package.source_query_completeness),package.bundle as i64,package.install_location_present.map(i64::from),json(&package.dependencies)?,package.permission_status,package.observed_at,format!("{:?}",package.current_user),format!("{:?}",package.other_users),format!("{:?}",package.provisioning)])?;
            }
        }
    }
    tx.execute("DELETE FROM desired_states", [])?;
    for desired in &store.desired {
        tx.execute("INSERT INTO desired_states(component_id,requested_state_json,scope,created_at,modified_at,selected_build,selected_edition,persistent_remediation,always_require_approval,user_note,validation_status,revision) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![desired.component_id.key(),json(&desired.state)?,desired.scope,desired.created_at,desired.modified_at,desired.selected_build,desired.selected_edition,desired.persistent as i64,desired.approval_required as i64,desired.note,desired.validation_status,desired.revision])?;
    }
    for revision in &store.desired_revisions {
        tx.execute("INSERT OR IGNORE INTO desired_state_revisions(component_id,revision,requested_state_json,scope,changed_at,persistent_remediation,always_require_approval,user_note,validation_status) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![revision.component_id.key(),revision.revision,json(&revision.state)?,revision.scope,revision.changed_at,revision.persistent as i64,revision.approval_required as i64,revision.note,revision.validation_status])?;
    }
    for event in &store.drift {
        tx.execute("INSERT INTO drift_events(component_id,previous_state_json,current_state_json,desired_state_json,classification,likely_cause,cause_confidence,first_detected,last_observed,resolved,occurrence_count,supporting_facts_json,alternative_causes_json,inference_rule_version,previous_inspection_id,current_inspection_id,reviewed,reviewed_at,returned_to_desired) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19) ON CONFLICT(component_id,classification,resolved) DO UPDATE SET current_state_json=excluded.current_state_json,last_observed=excluded.last_observed,occurrence_count=MAX(occurrence_count,excluded.occurrence_count),cause_confidence=excluded.cause_confidence,supporting_facts_json=excluded.supporting_facts_json,alternative_causes_json=excluded.alternative_causes_json,inference_rule_version=excluded.inference_rule_version,current_inspection_id=excluded.current_inspection_id,reviewed=excluded.reviewed,reviewed_at=excluded.reviewed_at,returned_to_desired=excluded.returned_to_desired",params![event.component_id.key(),json(&event.previous)?,json(&event.current)?,event.desired.as_ref().map(json).transpose()?,event.classification,event.cause,event.confidence,event.first_detected,event.last_observed,event.resolved as i64,event.occurrence_count,json(&event.supporting_facts)?,json(&event.alternative_causes)?,event.inference_rule_version,event.previous_inspection_id,event.current_inspection_id,event.reviewed as i64,event.reviewed_at,event.returned_to_desired as i64])?;
    }
    tx.execute(
        "INSERT INTO product_preferences(key,value) VALUES('history_retention_days',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [store.preferences.history_retention_days.to_string()],
    )?;
    Ok(())
}

fn management_summary(info: &PlatformInfo) -> String {
    format!(
        "domain_joined={}; workplace_joined={}; mdm_enrolled={}",
        info.domain_joined
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into()),
        info.workplace_joined
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into()),
        info.mdm_enrolled
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".into())
    )
}

fn load_store(conn: &Connection) -> rusqlite::Result<Store> {
    let mut store = Store {
        schema_version: SCHEMA_VERSION,
        database_status: DatabaseStatus::default(),
        ..Default::default()
    };
    store.preferences.history_retention_days = conn
        .query_row(
            "SELECT value FROM product_preferences WHERE key='history_retention_days'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .and_then(|value| value.parse().ok())
        .unwrap_or(180);
    let mut stmt = conn
        .prepare("SELECT id,started_at,platform_json,lifecycle_json,query_failures_json,machine_id FROM inspections ORDER BY completed_at")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))
    })?;
    for row in rows {
        let (id, timestamp, platform_json, lifecycle_json, query_failures_json, loaded_machine_id) =
            row?;
        let platform: PlatformInfo =
            serde_json::from_str(&platform_json).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let mut observations = vec![];
        let mut obs=conn.prepare("SELECT current_state_json,authority,applicable,evidence_json,observed_at,component_id,user_preference_state,policy_state,applicability_json,authority_attribution_json,control_precedence_json,detector_status,package_completeness,package_identities_json,warning_json,error_json,id FROM component_observations WHERE inspection_id=?1 ORDER BY id")?;
        let mapped = obs.query_map([&id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, String>(9)?,
                r.get::<_, String>(10)?,
                r.get::<_, String>(11)?,
                r.get::<_, String>(12)?,
                r.get::<_, String>(13)?,
                r.get::<_, String>(14)?,
                r.get::<_, Option<String>>(15)?,
                r.get::<_, i64>(16)?,
            ))
        })?;
        for item in mapped {
            let (
                state_json,
                authority,
                applicable,
                evidence_json,
                detected_at,
                key,
                preference_state,
                policy_state,
                applicability_json,
                authority_attribution_json,
                control_precedence_json,
                detector_status,
                package_completeness,
                package_identities_json,
                warning_json,
                error,
                observation_id,
            ) = item?;
            if let Some(component_id) = component_from_key(&key) {
                let packages = load_packages(conn, observation_id, component_id)?;
                observations.push(DetectionResult {
                    component_id,
                    current: serde_json::from_str(&state_json)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    authority: authority_from_str(&authority),
                    platform: platform.clone(),
                    applicable: applicable != 0,
                    applicability: serde_json::from_str(&applicability_json).unwrap_or_else(|_| {
                        crate::applicability::evaluate(component_id, &platform)
                    }),
                    evidence: serde_json::from_str(&evidence_json).unwrap_or_default(),
                    detected_at,
                    error,
                    warnings: serde_json::from_str(&warning_json).unwrap_or_default(),
                    package_identities: serde_json::from_str(&package_identities_json)
                        .unwrap_or_default(),
                    policy_state,
                    preference_state,
                    provisioning_state: None,
                    detector_status: detector_status_from_str(&detector_status),
                    authority_attribution: serde_json::from_str(&authority_attribution_json)
                        .unwrap_or_else(|_| crate::platform::AuthorityAttribution {
                            authority: authority_from_str(&authority),
                            ..Default::default()
                        }),
                    control_precedence: serde_json::from_str(&control_precedence_json)
                        .unwrap_or_default(),
                    package_completeness: package_completeness_from_str(&package_completeness),
                    packages,
                });
            }
        }
        store.snapshots.push(Snapshot {
            id,
            timestamp,
            platform,
            observations,
            lifecycle: lifecycle_json.and_then(|value| serde_json::from_str(&value).ok()),
            query_failures: serde_json::from_str(&query_failures_json).unwrap_or_default(),
            machine_id: loaded_machine_id,
        });
    }
    let mut stmt=conn.prepare("SELECT component_id,requested_state_json,scope,created_at,modified_at,persistent_remediation,always_require_approval,selected_build,selected_edition,user_note,validation_status,revision FROM desired_states ORDER BY component_id")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i64>(6)?,
            r.get::<_, u32>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, Option<String>>(9)?,
            r.get::<_, String>(10)?,
            r.get::<_, u32>(11)?,
        ))
    })?;
    for row in rows {
        let (
            key,
            state,
            scope,
            created_at,
            modified_at,
            persistent,
            approval_required,
            selected_build,
            selected_edition,
            note,
            validation_status,
            revision,
        ) = row?;
        if let Some(component_id) = component_from_key(&key) {
            store.desired.push(DesiredState {
                component_id,
                state: serde_json::from_str(&state).map_err(|_| rusqlite::Error::InvalidQuery)?,
                scope,
                created_at,
                modified_at,
                persistent: persistent != 0,
                approval_required: approval_required != 0,
                selected_build,
                selected_edition,
                note,
                validation_status,
                revision,
            });
        }
    }
    load_desired_revisions(conn, &mut store)?;
    let mut stmt=conn.prepare("SELECT component_id,previous_state_json,current_state_json,desired_state_json,classification,likely_cause,cause_confidence,first_detected,last_observed,resolved,occurrence_count,supporting_facts_json,alternative_causes_json,inference_rule_version,previous_inspection_id,current_inspection_id,reviewed,reviewed_at,returned_to_desired FROM drift_events ORDER BY first_detected")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, i64>(9)?,
            r.get::<_, u32>(10)?,
            r.get::<_, String>(11)?,
            r.get::<_, String>(12)?,
            r.get::<_, u32>(13)?,
            r.get::<_, Option<String>>(14)?,
            r.get::<_, Option<String>>(15)?,
            r.get::<_, i64>(16)?,
            r.get::<_, Option<String>>(17)?,
            r.get::<_, i64>(18)?,
        ))
    })?;
    for row in rows {
        let (
            key,
            previous,
            current,
            desired,
            classification,
            cause,
            confidence,
            first_detected,
            last_observed,
            resolved,
            occurrence_count,
            supporting_facts,
            alternative_causes,
            inference_rule_version,
            previous_inspection_id,
            current_inspection_id,
            reviewed,
            reviewed_at,
            returned_to_desired,
        ) = row?;
        if let Some(component_id) = component_from_key(&key) {
            store.drift.push(DriftEvent {
                component_id,
                previous: serde_json::from_str(&previous)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                current: serde_json::from_str(&current)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?,
                desired: desired
                    .map(|v| serde_json::from_str(&v).map_err(|_| rusqlite::Error::InvalidQuery))
                    .transpose()?,
                classification,
                cause,
                confidence,
                first_detected,
                last_observed,
                resolved: resolved != 0,
                reviewed: reviewed != 0,
                reviewed_at,
                returned_to_desired: returned_to_desired != 0,
                occurrence_count,
                supporting_facts: serde_json::from_str(&supporting_facts).unwrap_or_default(),
                alternative_causes: serde_json::from_str(&alternative_causes).unwrap_or_default(),
                inference_rule_version,
                previous_inspection_id,
                current_inspection_id,
            });
        }
    }
    Ok(store)
}

fn json<T: Serialize>(value: &T) -> rusqlite::Result<String> {
    serde_json::to_string(value).map_err(|_| rusqlite::Error::InvalidQuery)
}

fn load_packages(
    conn: &Connection,
    observation_id: i64,
    component_id: ComponentId,
) -> rusqlite::Result<Vec<crate::platform::PackageObservation>> {
    let mut statement = conn.prepare("SELECT package_family_name,package_full_name,package_name,version,architecture,publisher,current_user_state,other_user_state,provisioning_state,framework,resource_package,bundle,non_removable,install_location_present,dependencies_json,detection_completeness,permission_status,observed_at FROM package_observations WHERE observation_id=?1 ORDER BY package_name,package_full_name")?;
    let rows = statement.query_map([observation_id], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)?,
            row.get::<_, i64>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, i64>(12)?,
            row.get::<_, Option<i64>>(13)?,
            row.get::<_, String>(14)?,
            row.get::<_, String>(15)?,
            row.get::<_, String>(16)?,
            row.get::<_, Option<String>>(17)?,
        ))
    })?;
    let mut packages = Vec::new();
    for row in rows {
        let (
            family,
            full,
            name,
            version,
            architecture,
            publisher,
            current,
            other,
            provisioning,
            framework,
            resource,
            bundle,
            non_removable,
            location,
            dependencies,
            completeness,
            permission,
            observed_at,
        ) = row?;
        packages.push(crate::platform::PackageObservation {
            component_id,
            package_family_name: family,
            package_full_name: full,
            package_name: name.unwrap_or_default(),
            version,
            architecture,
            publisher_id: publisher,
            current_user: registration_from_str(&current),
            other_users: registration_from_str(&other),
            provisioning: provisioning_from_str(&provisioning),
            framework: framework != 0,
            resource_package: resource != 0,
            bundle: bundle != 0,
            non_removable: non_removable != 0,
            install_location_present: location.map(|value| value != 0),
            dependencies: serde_json::from_str(&dependencies).unwrap_or_default(),
            source_query_completeness: package_completeness_from_str(&completeness),
            permission_status: permission,
            observed_at: observed_at.unwrap_or_default(),
        });
    }
    Ok(packages)
}

fn load_desired_revisions(conn: &Connection, store: &mut Store) -> rusqlite::Result<()> {
    let mut statement = conn.prepare("SELECT component_id,revision,requested_state_json,scope,changed_at,persistent_remediation,always_require_approval,user_note,validation_status FROM desired_state_revisions ORDER BY component_id,revision")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, u32>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, String>(8)?,
        ))
    })?;
    for row in rows {
        let (
            key,
            revision,
            state,
            scope,
            changed_at,
            persistent,
            approval_required,
            note,
            validation_status,
        ) = row?;
        if let Some(component_id) = component_from_key(&key) {
            store.desired_revisions.push(DesiredStateRevision {
                component_id,
                revision,
                state: serde_json::from_str(&state).map_err(|_| rusqlite::Error::InvalidQuery)?,
                scope,
                changed_at,
                persistent: persistent != 0,
                approval_required: approval_required != 0,
                note,
                validation_status,
            });
        }
    }
    Ok(())
}

fn detector_status_from_str(value: &str) -> crate::platform::DetectorStatus {
    use crate::platform::DetectorStatus::*;
    match value.to_ascii_lowercase().as_str() {
        "successful" => Successful,
        "failed" => Failed,
        "cancelled" => Cancelled,
        "notrun" | "not_run" => NotRun,
        _ => Unknown,
    }
}

fn registration_from_str(value: &str) -> crate::platform::PackageRegistrationState {
    use crate::platform::PackageRegistrationState::*;
    match value.to_ascii_lowercase().as_str() {
        "present" => Present,
        "absent" => Absent,
        "permissionlimited" | "permission_limited" => PermissionLimited,
        "queryfailed" | "query_failed" => QueryFailed,
        "notapplicable" | "not_applicable" => NotApplicable,
        _ => Unknown,
    }
}

fn provisioning_from_str(value: &str) -> crate::platform::PackageProvisioningState {
    use crate::platform::PackageProvisioningState::*;
    match value.to_ascii_lowercase().as_str() {
        "provisioned" => Provisioned,
        "notprovisioned" | "not_provisioned" => NotProvisioned,
        "permissionlimited" | "permission_limited" => PermissionLimited,
        "queryfailed" | "query_failed" => QueryFailed,
        "notapplicable" | "not_applicable" => NotApplicable,
        _ => Unknown,
    }
}

fn package_completeness_from_str(value: &str) -> crate::platform::PackageCompleteness {
    use crate::platform::PackageCompleteness::*;
    match value.to_ascii_lowercase().replace('_', "").as_str() {
        "complete" => Complete,
        "currentuseronly" => CurrentUserOnly,
        "registrationcompleteprovisioningunknown" => RegistrationCompleteProvisioningUnknown,
        "provisioningcompleteallusersunknown" => ProvisioningCompleteAllUsersUnknown,
        "permissionlimited" => PermissionLimited,
        "failed" => Failed,
        "notapplicable" => NotApplicable,
        _ => Unknown,
    }
}
fn hash(value: &str) -> String {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    format!("{:016x}", h.finish())
}
fn machine_id(info: &PlatformInfo) -> String {
    #[cfg(feature = "owner-mode")]
    if let Some(scope) = info
        .owner_scope_id
        .as_deref()
        .filter(|scope| crate::owner_scope::is_valid(scope))
    {
        return scope.to_owned();
    }
    hash(&format!(
        "{}:{}:{}",
        info.device_name.as_deref().unwrap_or("unknown"),
        info.architecture,
        info.product_name
    ))
}

pub fn machine_identity(info: &PlatformInfo) -> String {
    machine_id(info)
}
fn component_from_key(key: &str) -> Option<ComponentId> {
    crate::platform::v1_catalogue()
        .into_iter()
        .find(|x| x.id.key() == key)
        .map(|x| x.id)
}
fn authority_from_str(value: &str) -> crate::platform::Authority {
    use crate::platform::Authority::*;
    match value {
        "Firmware" => Firmware,
        "Oem" => Oem,
        "DomainPolicy" => DomainPolicy,
        "Mdm" => Mdm,
        "LocalPolicy" => LocalPolicy,
        "SecurityProduct" => SecurityProduct,
        "WindowsServicing" => WindowsServicing,
        "Deslopper" => Deslopper,
        "User" => User,
        _ => Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{
        Authority, AuthorityAttribution, ControlPrecedence, DetectorStatus, Evidence,
        PackageCompleteness, PackageObservation, PackageProvisioningState,
        PackageRegistrationState,
    };

    fn temp_path(label: &str, extension: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "deslopper-{label}-{}-{nonce}.{extension}",
            std::process::id()
        ))
    }

    fn fixture_store() -> Store {
        let mut platform = crate::inspection::default_platform();
        platform.product_name = "Windows 11 Pro".into();
        platform.edition = "Professional".into();
        platform.build = 26100;
        platform.device_name = Some("TEST-PC".into());
        let package = PackageObservation {
            component_id: ComponentId::ConsumerCopilot,
            package_family_name: Some("Microsoft.Copilot_8wekyb3d8bbwe".into()),
            package_full_name: Some("Microsoft.Copilot_1.0.0.0_x64__8wekyb3d8bbwe".into()),
            package_name: "Microsoft.Copilot".into(),
            version: Some("1.0.0.0".into()),
            architecture: Some("X64".into()),
            publisher_id: Some("8wekyb3d8bbwe".into()),
            current_user: PackageRegistrationState::Present,
            other_users: PackageRegistrationState::PermissionLimited,
            provisioning: PackageProvisioningState::NotProvisioned,
            framework: false,
            resource_package: false,
            bundle: false,
            non_removable: false,
            install_location_present: Some(true),
            dependencies: vec!["Microsoft.VCLibs".into()],
            source_query_completeness: PackageCompleteness::PermissionLimited,
            permission_status: "all-user permission limited".into(),
            observed_at: "1001".into(),
        };
        let observation = DetectionResult {
            component_id: ComponentId::ConsumerCopilot,
            current: State::Package {
                current_user: PackageRegistrationState::Present,
                all_users: PackageRegistrationState::PermissionLimited,
                provisioned: PackageProvisioningState::NotProvisioned,
                version: Some("1.0.0.0".into()),
            },
            authority: Authority::WindowsServicing,
            platform: platform.clone(),
            applicable: true,
            applicability: crate::applicability::evaluate(ComponentId::ConsumerCopilot, &platform),
            evidence: vec![Evidence {
                query_id: "appx_current_user".into(),
                source: "fixture".into(),
                detail: "Exact identity observed".into(),
                confidence: 90,
            }],
            detected_at: "1001".into(),
            error: None,
            warnings: vec!["All-user scope permission limited".into()],
            package_identities: vec!["Microsoft.Copilot".into()],
            policy_state: None,
            preference_state: None,
            provisioning_state: Some("NotProvisioned".into()),
            detector_status: DetectorStatus::Unknown,
            authority_attribution: AuthorityAttribution {
                authority: Authority::WindowsServicing,
                ..Default::default()
            },
            control_precedence: ControlPrecedence::default(),
            package_completeness: PackageCompleteness::PermissionLimited,
            packages: vec![package],
        };
        let mut lifecycle = InspectionLifecycle::new("inspection-fixture".into(), 20);
        lifecycle.completed_detector_count = 20;
        lifecycle.successful_detector_count = 19;
        lifecycle.unknown_detector_count = 1;
        lifecycle.completed_at = Some("1010".into());
        lifecycle.phase = crate::inspection::InspectionPhase::CompletedWithPartialFailures;
        Store {
            schema_version: SCHEMA_VERSION,
            snapshots: vec![Snapshot {
                id: "inspection-fixture".into(),
                timestamp: "1000".into(),
                platform: platform.clone(),
                observations: vec![observation],
                lifecycle: Some(lifecycle),
                query_failures: vec![],
                machine_id: machine_identity(&platform),
            }],
            ..Default::default()
        }
    }
    #[test]
    fn schema_contains_all_beta_tables() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        for table in [
            "machines",
            "inspections",
            "component_observations",
            "package_observations",
            "desired_states",
            "drift_events",
            "preview_plans",
            "desired_state_revisions",
            "mutation_plans",
            "mutation_transactions",
            "mutation_steps",
            "mutation_state_captures",
            "mutation_rollbacks",
            "product_preferences",
        ] {
            let found: Option<String> = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .optional()
                .unwrap();
            assert_eq!(found.as_deref(), Some(table));
        }
    }

    #[test]
    fn fresh_database_reaches_latest_version_with_hardening() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let version: String = conn
            .query_row(
                "SELECT value FROM database_metadata WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION.to_string());
        assert_eq!(foreign_keys, 1);
        assert!(column_exists(&conn, "inspections", "lifecycle_json").unwrap());
    }

    #[test]
    fn version_one_upgrades_through_every_schema_version() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "UPDATE database_metadata SET value='1' WHERE key='schema_version'",
            [],
        )
        .unwrap();
        migrate(&conn).unwrap();
        assert!(column_exists(&conn, "package_observations", "permission_status").unwrap());
        let version: String = conn
            .query_row(
                "SELECT value FROM database_metadata WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION.to_string());
        assert!(column_exists(&conn, "drift_events", "reviewed").unwrap());
        assert!(column_exists(&conn, "drift_events", "returned_to_desired").unwrap());
    }

    #[test]
    fn interrupted_version_one_upgrade_resumes_idempotently() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute_batch(
            "DROP TABLE desired_state_revisions;
             UPDATE database_metadata SET value='1' WHERE key='schema_version';",
        )
        .unwrap();

        migrate(&conn).unwrap();

        assert!(column_exists(&conn, "inspections", "lifecycle_json").unwrap());
        let revisions: Option<String> = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='desired_state_revisions'",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        assert_eq!(revisions.as_deref(), Some("desired_state_revisions"));
    }

    #[test]
    fn version_two_upgrades_to_mutation_journal_schema() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute_batch(
            "DROP TABLE mutation_rollbacks;
             DROP TABLE mutation_state_captures;
             DROP TABLE mutation_steps;
             DROP TABLE mutation_transactions;
             DROP TABLE mutation_plans;
             UPDATE database_metadata SET value='2' WHERE key='schema_version';",
        )
        .unwrap();
        migrate(&conn).unwrap();
        for table in [
            "mutation_plans",
            "mutation_transactions",
            "mutation_steps",
            "mutation_state_captures",
            "mutation_rollbacks",
        ] {
            let found: Option<String> = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            assert_eq!(found.as_deref(), Some(table));
        }
    }

    #[test]
    fn version_three_upgrades_to_product_preferences_without_losing_history() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "INSERT INTO drift_events(component_id,previous_state_json,current_state_json,classification,likely_cause,cause_confidence,first_detected,last_observed,resolved,occurrence_count) VALUES('onedrive','{}','{}','Preference','User action','Weak','1','2',0,1)",
            [],
        )
        .unwrap();
        conn.execute_batch(
            "DROP TABLE product_preferences;
             UPDATE database_metadata SET value='3' WHERE key='schema_version';",
        )
        .unwrap();

        migrate(&conn).unwrap();

        let drift_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM drift_events", [], |row| row.get(0))
            .unwrap();
        let retention: String = conn
            .query_row(
                "SELECT value FROM product_preferences WHERE key='history_retention_days'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(drift_count, 1);
        assert_eq!(retention, "180");
        assert!(column_exists(&conn, "drift_events", "reviewed_at").unwrap());
    }

    #[test]
    fn version_four_upgrade_preserves_mutation_history_and_adds_terminal_noop_status() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute("INSERT INTO mutation_plans(id,machine_id,source_inspection_id,source_observation_id,component_id,operation_id,generated_at,expires_at,plan_hash,approval_nonce_hash,plan_json) VALUES('p','m','i','o','c','op','1','2','h','n','{}')",[]).unwrap();
        conn.execute("INSERT INTO mutation_transactions(id,plan_id,machine_id,component_id,operation_id,source_inspection_id,source_observation_id,created_at,status,required_privilege,handler_version,application_version,windows_build,windows_edition,plan_hash,transaction_json) VALUES('t','p','m','c','op','i','o','1','applied','user','v','v',26100,'Pro','h','{}')",[]).unwrap();
        conn.execute(
            "UPDATE database_metadata SET value='4' WHERE key='schema_version'",
            [],
        )
        .unwrap();

        migrate(&conn).unwrap();

        let retained: i64 = conn
            .query_row("SELECT COUNT(*) FROM mutation_transactions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(retained, 1);
        conn.execute(
            "UPDATE mutation_transactions SET status='no_change_needed' WHERE id='t'",
            [],
        )
        .unwrap();
        let foreign_key_errors: i64 = conn
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(foreign_key_errors, 0);
    }

    #[test]
    fn retention_keeps_recent_and_latest_snapshots_without_touching_desired_state() {
        let mut store = fixture_store();
        let mut old = store.snapshots[0].clone();
        old.id = "old-inspection".into();
        old.timestamp = "1000".into();
        let mut recent = store.snapshots[0].clone();
        recent.id = "recent-inspection".into();
        recent.timestamp = (40_u128 * 86_400_000).to_string();
        store.snapshots = vec![old, recent];
        store.preferences.history_retention_days = 30;
        let desired_count = store.desired.len();

        retain_history(&mut store, 45_u128 * 86_400_000);

        assert_eq!(store.snapshots.len(), 1);
        assert_eq!(store.snapshots[0].id, "recent-inspection");
        assert_eq!(store.desired.len(), desired_count);
    }

    #[test]
    fn clearing_read_only_data_preserves_internal_mutation_audit_tables() {
        let target = temp_path("clear-history", "db");
        let conn = open_at(&target).unwrap();
        conn.execute("INSERT INTO mutation_plans(id,machine_id,source_inspection_id,source_observation_id,component_id,operation_id,generated_at,expires_at,plan_hash,approval_nonce_hash,plan_json) VALUES('plan','machine','inspection','observation','component','operation','1','2','hash','nonce','{}')",[]).unwrap();
        drop(conn);
        let mut store = fixture_store();
        replace_read_only_data_at(&store, &target).unwrap();
        store.snapshots.clear();
        store.desired.clear();
        store.desired_revisions.clear();
        store.drift.clear();
        replace_read_only_data_at(&store, &target).unwrap();

        let conn = Connection::open(&target).unwrap();
        let inspections: i64 = conn
            .query_row("SELECT COUNT(*) FROM inspections", [], |row| row.get(0))
            .unwrap();
        let mutation_plans: i64 = conn
            .query_row("SELECT COUNT(*) FROM mutation_plans", [], |row| row.get(0))
            .unwrap();
        assert_eq!(inspections, 0);
        assert_eq!(mutation_plans, 1);
        drop(conn);
        let _ = fs::remove_file(target);
    }

    #[test]
    fn mutation_transaction_status_constraint_rejects_unknown_state() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute("INSERT INTO mutation_plans(id,machine_id,source_inspection_id,source_observation_id,component_id,operation_id,generated_at,expires_at,plan_hash,approval_nonce_hash,plan_json) VALUES('p','m','i','o','c','op','1','2','h','n','{}')",[]).unwrap();
        let result = conn.execute("INSERT INTO mutation_transactions(id,plan_id,machine_id,component_id,operation_id,source_inspection_id,source_observation_id,created_at,status,required_privilege,handler_version,application_version,windows_build,windows_edition,plan_hash,transaction_json) VALUES('t','p','m','c','op','i','o','1','not_a_state','user','v','v',26100,'Pro','h','{}')",[]);
        assert!(result.is_err());
    }

    #[test]
    fn database_lock_is_reported_without_recreating_the_database() {
        let path = temp_path("locked", "db");
        let owner = Connection::open(&path).unwrap();
        migrate(&owner).unwrap();
        owner.execute_batch("BEGIN EXCLUSIVE;").unwrap();

        let contender = Connection::open(&path).unwrap();
        contender
            .busy_timeout(std::time::Duration::from_millis(5))
            .unwrap();
        assert!(migrate(&contender).is_err());

        owner.execute_batch("ROLLBACK;").unwrap();
        drop(contender);
        drop(owner);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn newer_schema_is_rejected_without_recreation() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute(
            "UPDATE database_metadata SET value='999' WHERE key='schema_version'",
            [],
        )
        .unwrap();
        assert!(migrate(&conn).is_err());
        let version: String = conn
            .query_row(
                "SELECT value FROM database_metadata WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "999");
    }

    #[test]
    fn package_rows_and_cancellation_lifecycle_round_trip() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let store = fixture_store();
        let tx = conn.transaction().unwrap();
        persist(&tx, &store).unwrap();
        tx.commit().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM package_observations", [], |row| {
                row.get(0)
            })
            .unwrap();
        let state: String = conn
            .query_row(
                "SELECT other_user_state FROM package_observations",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(state, "PermissionLimited");
        let loaded = load_store(&conn).unwrap();
        assert_eq!(loaded.snapshots[0].observations[0].packages.len(), 1);
        assert_eq!(loaded.snapshots[0].timestamp, "1000");
        assert_eq!(
            loaded.snapshots[0]
                .lifecycle
                .as_ref()
                .map(|value| value.unknown_detector_count),
            Some(1)
        );
    }

    #[test]
    fn desired_state_revisions_preserve_timestamps() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let mut store = fixture_store();
        store.desired_revisions.push(DesiredStateRevision {
            component_id: ComponentId::ConsumerCopilot,
            revision: 1,
            state: State::UserPreference { enabled: false },
            scope: "current_user".into(),
            changed_at: "4242".into(),
            persistent: false,
            approval_required: true,
            note: None,
            validation_status: "valid".into(),
        });
        let tx = conn.transaction().unwrap();
        persist(&tx, &store).unwrap();
        tx.commit().unwrap();
        let loaded = load_store(&conn).unwrap();
        assert_eq!(loaded.desired_revisions[0].changed_at, "4242");
    }

    #[test]
    fn drift_history_preserves_first_and_last_observed_timestamps() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let mut store = fixture_store();
        store.drift.push(DriftEvent {
            component_id: ComponentId::ConsumerCopilot,
            previous: State::UserPreference { enabled: false },
            current: State::UserPreference { enabled: true },
            classification: "unexpected_change".into(),
            cause: "servicing_or_user_change".into(),
            confidence: "Medium".into(),
            first_detected: "1111".into(),
            last_observed: "2222".into(),
            occurrence_count: 3,
            supporting_facts: vec!["preference changed".into()],
            alternative_causes: vec!["manual change".into()],
            inference_rule_version: 2,
            previous_inspection_id: Some("inspection-old".into()),
            current_inspection_id: Some("inspection-fixture".into()),
            ..Default::default()
        });
        let tx = conn.transaction().unwrap();
        persist(&tx, &store).unwrap();
        tx.commit().unwrap();

        let loaded = load_store(&conn).unwrap();
        assert_eq!(loaded.drift[0].first_detected, "1111");
        assert_eq!(loaded.drift[0].last_observed, "2222");
        assert_eq!(loaded.drift[0].occurrence_count, 3);
    }

    #[test]
    fn alpha_parser_accepts_empty_and_partially_valid_json() {
        let empty = parse_alpha_store("{}").unwrap();
        assert!(empty.snapshots.is_empty());
        let valid_snapshot = serde_json::to_value(&fixture_store().snapshots[0]).unwrap();
        let partial = serde_json::json!({"schema_version":1,"snapshots":[valid_snapshot,{"bad":true}],"desired":[42]});
        let parsed = parse_alpha_store(&partial.to_string()).unwrap();
        assert_eq!(parsed.snapshots.len(), 1);
        assert!(parsed.desired.is_empty());
        assert!(parse_alpha_store("{malformed").is_err());
    }

    #[test]
    fn failed_import_transaction_preserves_raw_json() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute_batch("CREATE TRIGGER fail_import BEFORE INSERT ON inspections BEGIN SELECT RAISE(ABORT,'simulated write failure'); END;").unwrap();
        let path = std::env::temp_dir().join(format!(
            "deslopper-import-{}.json",
            crate::inspection::timestamp()
        ));
        fs::write(&path, serde_json::to_vec(&fixture_store()).unwrap()).unwrap();
        assert!(import_json_from(&conn, &path).is_err());
        assert!(path.exists());
        assert!(!path.with_extension("json.migrated.bak").exists());
    }

    #[test]
    fn alpha_import_handles_backup_collisions_and_prevents_duplicates() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let path = temp_path("import-collision", "json");
        let first_backup = path.with_extension("json.migrated.bak");
        let numbered_backup = path.with_extension("json.migrated.1.bak");
        let raw = serde_json::to_vec(&fixture_store()).unwrap();
        fs::write(&path, &raw).unwrap();
        fs::write(&first_backup, b"existing backup").unwrap();

        import_json_from(&conn, &path).unwrap();
        assert!(!path.exists());
        assert!(numbered_backup.exists());
        let inspections: i64 = conn
            .query_row("SELECT COUNT(*) FROM inspections", [], |row| row.get(0))
            .unwrap();
        assert_eq!(inspections, 1);

        fs::write(&path, &raw).unwrap();
        import_json_from(&conn, &path).unwrap();
        assert!(path.exists());
        let inspections_after_retry: i64 = conn
            .query_row("SELECT COUNT(*) FROM inspections", [], |row| row.get(0))
            .unwrap();
        assert_eq!(inspections_after_retry, 1);

        for candidate in [path, first_backup, numbered_backup] {
            let _ = fs::remove_file(candidate);
        }
    }

    #[test]
    fn malformed_alpha_import_is_retryable_and_preserves_the_source() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let path = temp_path("malformed-import", "json");
        fs::write(&path, b"{malformed").unwrap();

        import_json_from(&conn, &path).unwrap();

        let status: String = conn
            .query_row(
                "SELECT status FROM migration_log WHERE name='alpha_json'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "failed");
        assert!(path.exists());
        assert!(!path.with_extension("json.migrated.bak").exists());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn persistence_write_failure_rolls_back_without_partial_rows() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn.execute_batch("PRAGMA query_only=ON;").unwrap();
        let tx = conn.transaction().unwrap();
        assert!(persist(&tx, &fixture_store()).is_err());
        drop(tx);
        conn.execute_batch("PRAGMA query_only=OFF;").unwrap();
        let inspections: i64 = conn
            .query_row("SELECT COUNT(*) FROM inspections", [], |row| row.get(0))
            .unwrap();
        assert_eq!(inspections, 0);
    }

    #[test]
    fn privacy_redaction_applies_to_platform_and_structured_evidence() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let mut store = fixture_store();
        let profile =
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Sensitive".into());
        store.snapshots[0].platform.user_sid = Some("S-1-5-sensitive".into());
        store.snapshots[0].observations[0].evidence[0].detail =
            format!("Observed {profile}\\Documents");
        let tx = conn.transaction().unwrap();
        persist(&tx, &store).unwrap();
        tx.commit().unwrap();
        let platform_json: String = conn
            .query_row("SELECT platform_json FROM inspections", [], |row| {
                row.get(0)
            })
            .unwrap();
        let evidence_json: String = conn
            .query_row(
                "SELECT evidence_json FROM component_observations",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!platform_json.contains("S-1-5-sensitive"));
        assert!(!evidence_json.contains(&profile));
    }
    #[test]
    fn typed_state_json_round_trip() {
        let state = State::Package {
            current_user: crate::platform::PackageRegistrationState::Absent,
            all_users: crate::platform::PackageRegistrationState::Absent,
            provisioned: crate::platform::PackageProvisioningState::Provisioned,
            version: None,
        };
        let encoded = json(&state).unwrap();
        let decoded: State = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, state);
    }
}
