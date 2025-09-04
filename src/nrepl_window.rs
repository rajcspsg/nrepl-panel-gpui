use gpui::*;

use crate::chat_panel::ChatPanel;
use crate::message_editor::MessageEditor;

pub struct NReplWindow {
    chat_panel: Entity<ChatPanel>,
    message_editor: Entity<MessageEditor>,
}

impl NReplWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let chat_panel = ChatPanel::new(cx);
        let message_editor = cx.new(|cx| MessageEditor::new(window, cx));
        Self {
            chat_panel,
            message_editor,
        }
    }
}

impl Render for NReplWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().size_full().children(vec![
            div().flex_none().child(self.chat_panel.clone()),
            div().flex_grow().child(self.message_editor.clone()),
        ])
    }
}
