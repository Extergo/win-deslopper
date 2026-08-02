//! Deslopper's small composition root.

mod app;
mod applicability;
mod inspection;
mod model;
#[cfg(feature = "mutation-alpha")]
mod mutation;
mod package_identity;
mod persistence;
mod platform;
mod presentation;
mod privacy;
#[cfg(test)]
mod validation;

fn main() -> tauri::Result<()> {
    app::run()
}
