//! Serializable view models at the Rust-to-Svelte presentation boundary.

use serde::Serialize;

use crate::model::{AppState, CleanupComponent, ComponentId};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItemView {
    id: &'static str,
    icon_label: &'static str,
    name: &'static str,
    description: &'static str,
    category: &'static str,
    status: &'static str,
    risk: &'static str,
    restart: &'static str,
    compatibility: &'static str,
    proposed_actions: &'static str,
    planned: bool,
}

impl CleanupItemView {
    fn new(component: &CleanupComponent, state: &AppState) -> Self {
        Self {
            id: component.id.key(),
            icon_label: match component.id {
                ComponentId::OneDrive => "O",
                ComponentId::Copilot => "C",
                ComponentId::PromotionalContent => "P",
            },
            name: component.name,
            description: component.description,
            category: component.category.label(),
            status: "Preview only",
            risk: component.risk.label(),
            restart: component.restart.label(),
            compatibility: component.compatibility.label(),
            proposed_actions: component.proposed_actions,
            planned: state.is_planned(component.id),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppView {
    cleanup_items: Vec<CleanupItemView>,
    planned_items: Vec<CleanupItemView>,
    catalogue_count: usize,
    planned_count: usize,
    selected_filter: &'static str,
    selected_section: &'static str,
    review_open: bool,
}

impl From<&AppState> for AppView {
    fn from(state: &AppState) -> Self {
        let cleanup_items = state
            .visible_components()
            .into_iter()
            .map(|component| CleanupItemView::new(component, state))
            .collect();
        let planned_items = state
            .planned_components()
            .into_iter()
            .map(|component| CleanupItemView::new(component, state))
            .collect();

        Self {
            cleanup_items,
            planned_items,
            catalogue_count: state.catalogue_len(),
            planned_count: state.planned_count(),
            selected_filter: state.filter().key(),
            selected_section: state.destination().key(),
            review_open: state.is_review_open(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_model_reflects_filtered_and_planned_state() {
        let mut state = AppState::new();
        state.set_query("onedrive");
        state.toggle_planned(ComponentId::Copilot);

        let view = AppView::from(&state);

        assert_eq!(view.cleanup_items.len(), 1);
        assert_eq!(view.cleanup_items[0].id, "onedrive");
        assert_eq!(view.planned_items.len(), 1);
        assert_eq!(view.planned_items[0].id, "copilot");
        assert_eq!(view.planned_count, 1);
    }
}
