//! Tauri command wiring and local application-state orchestration.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    model::{AppState, CatalogueFilter, ComponentId, NavigationDestination},
    presentation::AppView,
};

pub struct ManagedAppState(Mutex<AppState>);

impl ManagedAppState {
    fn new() -> Self {
        Self(Mutex::new(AppState::new()))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum AppAction {
    SearchChanged { query: String },
    FilterChanged { filter: String },
    TogglePlanned { component_id: String },
    Navigate { destination: String },
    OpenReview,
    CloseReview,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    code: &'static str,
    message: String,
}

impl CommandError {
    fn invalid_action(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_action",
            message: message.into(),
        }
    }

    fn state_unavailable() -> Self {
        Self {
            code: "state_unavailable",
            message: "The local preview state is temporarily unavailable.".to_owned(),
        }
    }
}

#[tauri::command]
pub fn get_app_view(state: State<'_, ManagedAppState>) -> Result<AppView, CommandError> {
    let state = state
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    Ok(AppView::from(&*state))
}

#[tauri::command]
pub fn dispatch_app_action(
    action: AppAction,
    state: State<'_, ManagedAppState>,
) -> Result<AppView, CommandError> {
    let mut state = state
        .0
        .lock()
        .map_err(|_| CommandError::state_unavailable())?;
    apply_action(&mut state, action)?;
    Ok(AppView::from(&*state))
}

fn apply_action(state: &mut AppState, action: AppAction) -> Result<(), CommandError> {
    match action {
        AppAction::SearchChanged { query } => state.set_query(query),
        AppAction::FilterChanged { filter } => {
            let filter = CatalogueFilter::from_key(&filter).ok_or_else(|| {
                CommandError::invalid_action(format!("Unknown catalogue filter: {filter}"))
            })?;
            state.set_filter(filter);
        }
        AppAction::TogglePlanned { component_id } => {
            let component_id = ComponentId::from_key(&component_id).ok_or_else(|| {
                CommandError::invalid_action(format!("Unknown component: {component_id}"))
            })?;
            state.toggle_planned(component_id);
        }
        AppAction::Navigate { destination } => {
            let destination = NavigationDestination::from_key(&destination).ok_or_else(|| {
                CommandError::invalid_action(format!(
                    "Unknown navigation destination: {destination}"
                ))
            })?;
            state.set_destination(destination);
        }
        AppAction::OpenReview => state.open_review(),
        AppAction::CloseReview => state.close_review(),
    }

    Ok(())
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(ManagedAppState::new())
        .invoke_handler(tauri::generate_handler![get_app_view, dispatch_app_action])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_update_rust_owned_preview_state() {
        let mut state = AppState::new();

        apply_action(
            &mut state,
            AppAction::SearchChanged {
                query: "copilot".to_owned(),
            },
        )
        .expect("valid search action should be accepted");
        assert_eq!(state.visible_components().len(), 1);

        apply_action(
            &mut state,
            AppAction::TogglePlanned {
                component_id: "copilot".to_owned(),
            },
        )
        .expect("known component should be accepted");
        assert_eq!(state.planned_count(), 1);

        apply_action(&mut state, AppAction::OpenReview).expect("review action should be accepted");
        assert!(state.is_review_open());
    }

    #[test]
    fn invalid_transport_values_are_rejected() {
        let mut state = AppState::new();
        let error = apply_action(
            &mut state,
            AppAction::FilterChanged {
                filter: "everything".to_owned(),
            },
        )
        .expect_err("unknown filters must be rejected");

        assert_eq!(error.code, "invalid_action");
        assert_eq!(state.filter(), CatalogueFilter::All);
    }
}
