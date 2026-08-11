//! Deslopper's small composition root.

mod app;
mod applicability;
mod inspection;
mod model;
#[cfg(feature = "owner-mode")]
mod mutation;
#[cfg(feature = "owner-mode")]
mod owner_scope;
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
