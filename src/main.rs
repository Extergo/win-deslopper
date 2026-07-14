slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let app_weak = app.as_weak();

    app.on_inspect_system(move || {
        if let Some(app) = app_weak.upgrade() {
            app.set_status_text("Deslopper is running.".into());
        }
    });

    app.run()
}
