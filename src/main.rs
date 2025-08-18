mod agent_panel;
mod message_editor;
mod thread;

use agent_panel::*;
use gpui::*;
use std::sync::Arc;

fn main() {
    Application::new().run(|app| {
        settings::init(app);
        theme::init(theme::LoadThemes::JustBase, app);
        language::init(app);
        project::Project::init_settings(app);
        workspace::init_settings(app);
        editor::init(app);

        let fs: Arc<dyn project::Fs> = fs::FakeFs::new(app.background_executor().clone());
        app.open_window(WindowOptions::default(), move |window, cx| {
            cx.new(|cx| AgentPanel::new(fs.clone(), window, cx))
        })
        .unwrap();
    });
}
