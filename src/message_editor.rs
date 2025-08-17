use crate::thread::*;
use editor;
use editor::*;
use futures::future::Shared;
use gpui::*;
use settings::*;
use std::sync::Arc;
use std::time::Duration;
use theme::ThemeSettings;
//use tool_compatibility::{IncompatibleToolsState, IncompatibleToolsTooltip};
use language::{Buffer, Language, Point};
use ui::*;
use ui::{
    Callout, Disclosure, Divider, DividerColor, KeyBinding, PopoverMenuHandle, Tooltip, prelude::*,
};
use ui_macros::RegisterComponent;
use workspace::*;

pub const MIN_EDITOR_LINES: usize = 4;
pub const MAX_EDITOR_LINES: usize = 8;

pub struct MessageEditor {
    editor: Entity<Editor>,
    load_context_task: Option<Shared<Task<()>>>,
    edits_expanded: bool,
    editor_is_expanded: bool,
    last_estimated_token_count: Option<u64>,
    update_token_count_task: Option<Task<()>>,
    _subscriptions: Vec<gpui::Subscription>,
}

pub fn create_editor(
    min_lines: usize,
    max_lines: Option<usize>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Editor> {
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
        editor.set_placeholder_text("Message the agent – @ to include context", cx);
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
    pub fn new(thread: Entity<Thread>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = create_editor(MIN_EDITOR_LINES, Some(MAX_EDITOR_LINES), window, cx);

        let subscriptions = vec![cx.subscribe(&editor, |this, _, event, cx| match event {
            EditorEvent::BufferEdited => this.handle_message_changed(cx),
            _ => {}
        })];

        //let project = thread.read(cx).project().clone();

        Self {
            editor: editor.clone(),
            load_context_task: None,
            edits_expanded: false,
            editor_is_expanded: false,
            last_estimated_token_count: None,
            update_token_count_task: None,
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

    pub fn expand_message_editor(
        &mut self,
        // _: &ExpandMessageEditor,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
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
        // self.message_or_context_changed(true, cx);
    }

    fn message_or_context_changed(&mut self, debounce: bool, cx: &mut Context<Self>) {
        cx.emit(MessageEditorEvent::Changed);
        self.update_token_count_task.take();

        let editor = self.editor.clone();

        self.update_token_count_task = Some(cx.spawn(async move |this, cx| {
            if debounce {
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
            }

            let token_count = if let Some(task) = this
                .update(cx, |this, cx| {
                    let message_text = editor.read(cx).text(cx);

                    if message_text.is_empty() {
                        return None;
                    }
                    Some(1)
                })
                .ok()
                .flatten()
            {
                //task.await.log_err()
                Some(1)
            } else {
                Some(0)
            };

            this.update(cx, |this, cx| {
                if let Some(token_count) = token_count {
                    this.last_estimated_token_count = Some(token_count);
                    cx.emit(MessageEditorEvent::EstimatedTokenCount);
                }
                this.update_token_count_task.take();
            })
            .ok();
        }));
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
        //let incompatible_tools = cx.new(|cx| IncompatibleToolsState::new(thread.clone(), cx));

        v_flex()
            .key_context("MessageEditor")
            .p_2()
            .gap_2()
            .border_t_1()
            .border_color(cx.theme().colors().border)
            .bg(editor_bg_color)
            .child(
                h_flex()
                    .justify_between()
                    //.child(self.context_strip.clone())
                    .when(focus_handle.is_focused(window), |this| {
                        this.child(
                            IconButton::new("toggle-height", expand_icon)
                                .icon_size(IconSize::XSmall)
                                .icon_color(Color::Muted)
                                /*.tooltip({
                                    let focus_handle = focus_handle.clone();
                                    move |window, cx| {
                                        let expand_label = if is_editor_expanded {
                                            "Minimize Message Editor".to_string()
                                        } else {
                                            "Expand Message Editor".to_string()
                                        };
                                        //Tooltip::for_action_in(
                                        //    expand_label,
                                        //    &ExpandMessageEditor,
                                        //    &focus_handle,
                                        //    window,
                                        //    cx,
                                        //)
                                    }
                                }) */
                                .on_click(cx.listener(|_, _, window, cx| {
                                    //   window.dispatch_action(Box::new(ExpandMessageEditor), cx);
                                })),
                        )
                    }),
            )
            .child(
                v_flex()
                    .size_full()
                    .gap_1()
                    .when(is_editor_expanded, |this| {
                        this.h(vh(0.8, window)).justify_between()
                    })
                    .child({
                        let settings = ThemeSettings::get_global(cx);
                        let font_size = TextSize::Small
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
                    })
                    .child(
                        h_flex()
                            .flex_none()
                            .flex_wrap()
                            .justify_between()
                            .child(
                                h_flex(), // .child(self.render_follow_toggle(is_model_selected, cx))
                                          //  .children(self.render_burn_mode_toggle(cx)),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .flex_wrap()
                                    .when(true, |this| {
                                        this.child(
                                            IconButton::new(
                                                "tools-incompatible-warning",
                                                IconName::Warning,
                                            )
                                            .icon_color(Color::Warning)
                                            .icon_size(IconSize::Small), /*.tooltip({
                                                                             move |_, cx| {
                                                                                 cx.new(|_| IncompatibleToolsTooltip {
                                                                                     incompatible_tools: incompatible_tools
                                                                                         .clone(),
                                                                                 })
                                                                                 .into()
                                                                             }
                                                                         }), */
                                        )
                                    })
                                    .map({
                                        let focus_handle = focus_handle.clone();
                                        move |parent| {
                                            if true {
                                                parent
                                                    .when(is_editor_empty, |parent| {
                                                        parent.child(
                                                            IconButton::new(
                                                                "stop-generation",
                                                                IconName::StopFilled,
                                                            )
                                                            .icon_color(Color::Error)
                                                            .style(ButtonStyle::Tinted(
                                                                ui::TintColor::Error,
                                                            ))
                                                            .tooltip(move |window, cx| {
                                                                Tooltip::for_action(
                                                                    "Stop Generation",
                                                                    &editor::actions::Cancel,
                                                                    window,
                                                                    cx,
                                                                )
                                                            })
                                                            .on_click({
                                                                let focus_handle =
                                                                    focus_handle.clone();
                                                                move |_event, window, cx| {
                                                                    focus_handle.dispatch_action(
                                                                        &editor::actions::Cancel,
                                                                        window,
                                                                        cx,
                                                                    );
                                                                }
                                                            })
                                                            .with_animation(
                                                                "pulsating-label",
                                                                Animation::new(
                                                                    Duration::from_secs(2),
                                                                )
                                                                .repeat()
                                                                .with_easing(pulsating_between(
                                                                    0.4, 1.0,
                                                                )),
                                                                |icon_button, delta| {
                                                                    icon_button.alpha(delta)
                                                                },
                                                            ),
                                                        )
                                                    })
                                                    .when(!is_editor_empty, |parent| {
                                                        parent.child(
                                                            IconButton::new(
                                                                "send-message",
                                                                IconName::Send,
                                                            )
                                                            .icon_color(Color::Accent)
                                                            .style(ButtonStyle::Filled),
                                                        )
                                                    })
                                            } else {
                                                parent.child(
                                                    IconButton::new("send-message", IconName::Send)
                                                        .icon_color(Color::Accent)
                                                        .style(ButtonStyle::Filled)
                                                        .when(is_editor_empty, |button| {
                                                            button.tooltip(Tooltip::text(
                                                                "Type a message to submit",
                                                            ))
                                                        })
                                                        .when(!true, |button| {
                                                            button.tooltip(Tooltip::text(
                                                                "Select a model to continue",
                                                            ))
                                                        }),
                                                )
                                            }
                                        }
                                    }),
                            ),
                    ),
            )
    }
}

impl EventEmitter<MessageEditorEvent> for MessageEditor {}

pub enum MessageEditorEvent {
    EstimatedTokenCount,
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
        let line_height = TextSize::Small.rems(cx).to_pixels(window.rem_size()) * 1.5;

        v_flex()
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_editor(window, cx))
    }
}
