//! Deslopper's small composition root.

mod app;
mod model;
mod presentation;

fn main() -> tauri::Result<()> {
    app::run()
}
