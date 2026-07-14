//! UI callback wiring and local application-state orchestration.

use std::{cell::RefCell, rc::Rc};

use slint::ComponentHandle;

use crate::{
    AppWindow,
    model::{AppState, CatalogueFilter, ComponentId, NavigationDestination},
    ui_bridge,
};

pub fn run() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let state = Rc::new(RefCell::new(AppState::new()));

    refresh(&window, &state.borrow());
    wire_callbacks(&window, &state);

    window.run()
}

fn refresh(window: &AppWindow, state: &AppState) {
    window.set_cleanup_items(ui_bridge::visible_items(state));
    window.set_planned_items(ui_bridge::planned_items(state));
    window.set_catalogue_count(state.catalogue_len() as i32);
    window.set_planned_count(state.planned_count() as i32);
    window.set_selected_filter(state.filter().index());
    window.set_selected_section(state.destination().index());
    window.set_review_open(state.is_review_open());
}

fn wire_callbacks(window: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let window_weak = window.as_weak();
    let shared_state = Rc::clone(state);
    window.on_search_changed(move |query| {
        shared_state.borrow_mut().set_query(query.as_str());
        if let Some(window) = window_weak.upgrade() {
            refresh(&window, &shared_state.borrow());
        }
    });

    let window_weak = window.as_weak();
    let shared_state = Rc::clone(state);
    window.on_filter_changed(move |index| {
        shared_state
            .borrow_mut()
            .set_filter(CatalogueFilter::from_index(index));
        if let Some(window) = window_weak.upgrade() {
            refresh(&window, &shared_state.borrow());
        }
    });

    let window_weak = window.as_weak();
    let shared_state = Rc::clone(state);
    window.on_toggle_plan(move |index| {
        if let Some(id) = ComponentId::from_index(index) {
            shared_state.borrow_mut().toggle_planned(id);
            if let Some(window) = window_weak.upgrade() {
                refresh(&window, &shared_state.borrow());
            }
        }
    });

    let window_weak = window.as_weak();
    let shared_state = Rc::clone(state);
    window.on_navigate(move |index| {
        shared_state
            .borrow_mut()
            .set_destination(NavigationDestination::from_index(index));
        if let Some(window) = window_weak.upgrade() {
            refresh(&window, &shared_state.borrow());
        }
    });

    let window_weak = window.as_weak();
    let shared_state = Rc::clone(state);
    window.on_open_review(move || {
        shared_state.borrow_mut().open_review();
        if let Some(window) = window_weak.upgrade() {
            refresh(&window, &shared_state.borrow());
        }
    });

    let window_weak = window.as_weak();
    let shared_state = Rc::clone(state);
    window.on_close_review(move || {
        shared_state.borrow_mut().close_review();
        if let Some(window) = window_weak.upgrade() {
            refresh(&window, &shared_state.borrow());
        }
    });
}
