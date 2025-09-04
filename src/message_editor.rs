use crate::nrepl_client::*;
use crate::state::*;
use editor::*;
use gpui::*;
use language::Buffer;
use settings::*;
use std::sync::Arc;
use theme::ThemeSettings;
use ui::*;
use workspace::*;

pub const MIN_EDITOR_LINES: usize = 4;
pub const MAX_EDITOR_LINES: usize = 8;

pub struct MessageEditor {
    pub editor: Entity<Editor>,
    pub editor_is_expanded: bool,
    pub _subscriptions: Vec<gpui::Subscription>,
}

pub fn create_editor(window: &mut Window, cx: &mut App) -> Entity<Editor> {
    let editor = cx.new(|cx| {
        let buffer = cx.new(|cx| Buffer::local("", cx));
        let buffer = cx.new(|cx| MultiBuffer::singleton(buffer, cx));
        let mut editor = Editor::new(
            editor::EditorMode::AutoHeight {
                min_lines: 4,
                max_lines: Some(8),
            },
            buffer,
            None,
            window,
            cx,
        );
        editor.set_placeholder_text("Send clojure s-expressions to NRepl Server", cx);
        editor.set_show_indent_guides(false, cx);
        editor.set_soft_wrap();
        editor.set_use_modal_editing(true);
        editor.set_context_menu_options(ContextMenuOptions {
            min_entries_visible: 12,
            max_entries_visible: 12,
            placement: Some(ContextMenuPlacement::Above),
        });
        editor
    });

    //let editor_entity = editor.downgrade();

    editor
}

impl MessageEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = create_editor(window, cx);
        let focus_handle = editor.focus_handle(cx);
        focus_handle.focus(window);

        let subscriptions = vec![cx.subscribe(&editor, |this, _, event, cx| match event {
            EditorEvent::BufferEdited => {
                this.handle_message_changed(cx);
            }
            _ => {}
        })];

        Self {
            editor: editor.clone(),
            editor_is_expanded: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn get_text(&self, cx: &App) -> String {
        self.editor.read(cx).text(cx)
    }

    pub fn set_text(
        &mut self,
        text: impl Into<Arc<str>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editor.update(cx, |editor, cx| {
            editor.set_text(text, window, cx);
        });
    }

    pub fn expand_message_editor(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_editor_is_expanded(!self.editor_is_expanded, cx);
    }

    fn set_editor_is_expanded(&mut self, is_expanded: bool, cx: &mut Context<Self>) {
        self.editor_is_expanded = is_expanded;
        self.editor.update(cx, |editor, _| {
            if self.editor_is_expanded {
                editor.set_mode(EditorMode::Full {
                    scale_ui_elements_with_buffer_font_size: false,
                    show_active_line_background: false,
                    sized_by_content: false,
                })
            } else {
                editor.set_mode(EditorMode::AutoHeight {
                    min_lines: MIN_EDITOR_LINES,
                    max_lines: Some(MAX_EDITOR_LINES),
                })
            }
        });
        cx.notify();
    }

    fn is_editor_empty(&self, cx: &App) -> bool {
        self.editor.read(cx).text(cx).trim().is_empty()
    }

    pub fn is_editor_fully_empty(&self, cx: &App) -> bool {
        self.editor.read(cx).is_empty(cx)
    }

    fn handle_message_changed(&mut self, cx: &mut Context<Self>) {
        self.message_or_context_changed(true, cx);
    }

    fn message_or_context_changed(&mut self, debounce: bool, cx: &mut Context<Self>) {
        cx.emit(MessageEditorEvent::Changed);
    }

    fn render_editor(&self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let editor_bg_color = cx.theme().colors().editor_background;
        let focus_handle = self.editor.focus_handle(cx);

        let is_editor_empty = self.is_editor_empty(cx);

        let is_editor_expanded = self.editor_is_expanded;
        let expand_icon = if is_editor_expanded {
            IconName::Minimize
        } else {
            IconName::Maximize
        };

        v_flex()
            .key_context("MessageEditor")
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, _window, cx| {
                cx.propagate();
            }))
            .on_key_up(cx.listener(|_this, event: &KeyUpEvent, _window, cx| {
                cx.propagate();
            }))
            .p_2()
            .gap_2()
            .border_t_1()
            .border_color(cx.theme().colors().border)
            .bg(editor_bg_color)
            .child(h_flex().justify_between())
            .child(
                v_flex()
                    .size_full()
                    .gap_1()
                    .when(is_editor_expanded, |this| {
                        this.h(vh(0.8, window)).justify_between()
                    })
                    .child({
                        let settings = ThemeSettings::get_global(cx);
                        let font_size = TextSize::Default
                            .rems(cx)
                            .to_pixels(settings.agent_font_size(cx));
                        let line_height = settings.buffer_line_height.value() * font_size;

                        let text_style = TextStyle {
                            color: cx.theme().colors().text,
                            font_family: settings.buffer_font.family.clone(),
                            font_fallbacks: settings.buffer_font.fallbacks.clone(),
                            font_features: settings.buffer_font.features.clone(),
                            font_size: font_size.into(),
                            line_height: line_height.into(),
                            ..Default::default()
                        };

                        {
                            EditorElement::new(
                                &self.editor,
                                EditorStyle {
                                    background: editor_bg_color,
                                    local_player: cx.theme().players().local(),
                                    text: text_style,
                                    syntax: cx.theme().syntax().clone(),
                                    ..Default::default()
                                },
                            )
                            .into_any()
                        }
                    })
                    .child(
                        h_flex()
                            .flex_none()
                            .flex_wrap()
                            .justify_between()
                            .child(h_flex())
                            .child(h_flex().gap_1().flex_wrap().map({
                                let focus_handle = focus_handle.clone();
                                move |parent| {
                                    parent.child(
                                        IconButton::new("send-message", IconName::Send)
                                            .icon_color(Color::Accent)
                                            .style(ButtonStyle::Filled)
                                            .when(is_editor_empty, |button| {
                                                button.tooltip(Tooltip::text(
                                                    "Type a message to submit",
                                                ))
                                            })
                                            .on_click(cx.listener(|this, _, _window, cx| {
                                                let input_cmd = this.get_text(cx);
                                                StateModel::update(
                                                    |this, cx| {
                                                        let item = NreplRequest {
                                                            id: this.inner.clone().read(cx).count,
                                                            req: input_cmd.into(),
                                                        };
                                                        this.push(item, cx);
                                                    },
                                                    cx,
                                                );

                                                this.set_text("", _window, cx);
                                            })),
                                    )
                                }
                            })),
                    ),
            )
    }
}

impl EventEmitter<MessageEditorEvent> for MessageEditor {}

pub enum MessageEditorEvent {
    Changed,
    ScrollThreadToBottom,
}

impl Focusable for MessageEditor {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.editor.focus_handle(cx)
    }
}

impl Render for MessageEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut key_context = KeyContext::default();
        key_context.add("MessageEditor");

        v_flex()
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_editor(window, cx))
    }
}
