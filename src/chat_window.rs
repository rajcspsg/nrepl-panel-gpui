use gpui::*;

use crate::chat_panel::ChatPanel;
use crate::message_editor::MessageEditor;
use crate::thread::Thread;

/// The ChatWindow panel displays the chat messages at the top and the message editor below.
pub struct ChatWindow {
    chat_panel: Entity<ChatPanel>,
    message_editor: Entity<MessageEditor>,
}

impl ChatWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // For demonstration, create a dummy thread and demo messages.
        let thread = cx.new(|_| Thread::new("demo-thread"));
        let chat_panel = cx.new(|_| ChatPanel::demo());
        let message_editor = cx.new(|cx| MessageEditor::new(thread, window, cx));
        Self {
            chat_panel,
            message_editor,
        }
    }
}

impl Render for ChatWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Layout: ChatPanel at the top, MessageEditor at the bottom.
        div().flex().flex_col().size_full().children(vec![
            div().flex_none().child(self.chat_panel.clone()),
            div().flex_grow().child(self.message_editor.clone()),
        ])
    }
}
