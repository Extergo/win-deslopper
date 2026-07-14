//! Conversion between domain-owned data and Slint presentation models.

use std::rc::Rc;

use slint::{ModelRc, SharedString, VecModel};

use crate::{CleanupItemView, model::AppState};

pub fn visible_items(state: &AppState) -> ModelRc<CleanupItemView> {
    model_from(state, false)
}

pub fn planned_items(state: &AppState) -> ModelRc<CleanupItemView> {
    model_from(state, true)
}

fn model_from(state: &AppState, planned_only: bool) -> ModelRc<CleanupItemView> {
    let components = if planned_only {
        state.planned_components()
    } else {
        state.visible_components()
    };
    let items = components
        .into_iter()
        .map(|component| CleanupItemView {
            id: component.id.index(),
            icon_label: SharedString::from(match component.id {
                crate::model::ComponentId::OneDrive => "O",
                crate::model::ComponentId::Copilot => "C",
                crate::model::ComponentId::PromotionalContent => "P",
            }),
            name: SharedString::from(component.name),
            description: SharedString::from(component.description),
            category: SharedString::from(component.category.label()),
            status: SharedString::from("Preview only"),
            risk: SharedString::from(component.risk.label()),
            restart: SharedString::from(component.restart.label()),
            compatibility: SharedString::from(component.compatibility.label()),
            proposed_actions: SharedString::from(component.proposed_actions),
            planned: state.is_planned(component.id),
        })
        .collect::<Vec<_>>();

    ModelRc::from(Rc::new(VecModel::from(items)))
}
