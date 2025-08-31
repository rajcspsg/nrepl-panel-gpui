use chat_panel::*;
use chat_window::*;
use colors::*;
use gpui::*;
use std::sync::Arc;
use text::*;
pub mod assets;
pub mod chat_panel;
pub mod chat_window;
pub mod icon;
pub mod message_editor;
pub mod nrepl_client;
pub mod state;
pub mod story;
pub mod text;
pub mod themes;
pub mod thread;

use project::Project;
use state::*;
use theme::*;
use themes::*;

fn main() {
    Application::new().run(move |cx| {
        cx.bind_keys([
            KeyBinding::new("backspace", message_editor::Backspace, None),
            KeyBinding::new("delete", message_editor::Delete, None),
            KeyBinding::new("left", message_editor::Left, None),
            KeyBinding::new("right", message_editor::Right, None),
            KeyBinding::new("shift-left", message_editor::SelectLeft, None),
            KeyBinding::new("shift-right", message_editor::SelectRight, None),
            KeyBinding::new("cmd-a", message_editor::SelectAll, None),
            KeyBinding::new("home", message_editor::Home, None),
            KeyBinding::new("end", message_editor::End, None),
            KeyBinding::new("ctrl-cmd-space", message_editor::ShowCharacterPalette, None),
            //KeyBinding::new("cmd-q", message_editor::Quit, None),
        ]);
        cx.activate(true);
        settings::init(cx);
        theme::init(theme::LoadThemes::All(Box::new(assets::Assets)), cx);
        language::init(cx);
        editor::init(cx);
        Project::init_settings(cx);
        workspace::init_settings(cx);
        cx.set_global(GlobalColors(Arc::new(Colors::default())));
        let size = size(px(1300.), px(500.));
        let bounds = Bounds::centered(None, size, cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                StateModel::init(cx, 63900);
                cx.new(|cx| ChatWindow::new(window, cx))
            },
        );
    });
}
