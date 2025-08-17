mod agent_panel;
mod message_editor;
mod thread;

use agent_panel::*;
use gpui::*;
use std::sync::Arc;
use thread::*;

fn main() {
    Application::new().run(|app| {
        settings::init(app);
        theme::init(theme::LoadThemes::JustBase, app);
        language::init(app);
        project::Project::init_settings(app);
        workspace::init_settings(app);
        editor::init(app);

        // Create minimal fake filesystem
        let fs = Arc::new(fs::FakeFs::new(app.background_executor().clone()));
        app.open_window(WindowOptions::default(), |_window, app| {
            app.new(|_cx| AgentPanel::new(fs, _window, app))
        })
        .unwrap();
    });
}
