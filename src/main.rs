//! Deslopper's small composition root.

mod app;
mod model;
mod ui_bridge;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    app::run()
}
