use colors::*;
use gpui::*;

#[derive(Clone)]
pub struct ChatMessage {
    pub author: SharedString,
    pub content: SharedString,
}

impl ChatMessage {
    pub fn new(author: &str, content: &str) -> Self {
        Self {
            author: author.to_owned().into(),
            content: content.to_owned().into(),
        }
    }
}

/// The ChatPanel component, which displays a list of chat messages.
pub struct ChatPanel {
    pub messages: Vec<ChatMessage>,
}

impl ChatPanel {
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self { messages }
    }

    pub fn demo() -> Self {
        Self::new(vec![
            ChatMessage::new("Alice", "Hello, how are you?"),
            ChatMessage::new("Bob", "I'm good, thanks! How about you?"),
            ChatMessage::new(
                "Alice",
                "Doing well. Ready to start our project discussion.",
            ),
            ChatMessage::new("Bob", "Absolutely! Let's get started."),
        ])
    }
}

impl Render for ChatPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Panel container
        let colors = Colors::default();
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.background)
            .children(
                self.messages
                    .iter()
                    .map(|msg| {
                        div()
                            .flex()
                            .flex_col()
                            .p_2()
                            .m_2()
                            .rounded_md()
                            .bg(colors.container)
                            .child(
                                div()
                                    .text_sm()
                                    //.text_color(colors.accent)
                                    .child(msg.author.clone()),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors.text)
                                    .child(msg.content.clone()),
                            )
                    })
                    .collect::<Vec<_>>(),
            )
    }
}
