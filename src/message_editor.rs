use crate::nrepl_client::*;
use crate::state::*;
use crate::thread::*;
use editor::*;
use gpui::*;
use language::Buffer;
use settings::*;
use std::ops::Range;
use std::sync::Arc;
use theme::ThemeSettings;
use ui::*;
use unicode_segmentation::*;
use workspace::*;

pub const MIN_EDITOR_LINES: usize = 4;
pub const MAX_EDITOR_LINES: usize = 8;

actions!(
    message_editor,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

pub struct MessageEditor {
    pub editor: Entity<Editor>,
    pub editor_is_expanded: bool,
    pub _subscriptions: Vec<gpui::Subscription>,
    pub content: SharedString,
    pub placeholder: SharedString,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<ShapedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
    pub is_selecting: bool,
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
            editor_is_expanded: false,
            _subscriptions: subscriptions,
            content: "".into(),
            placeholder: "Enter Clojure Expression...".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
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
        //println!("handle message changed called");
        self.message_or_context_changed(true, cx);
    }

    fn message_or_context_changed(&mut self, debounce: bool, cx: &mut Context<Self>) {
        cx.emit(MessageEditorEvent::Changed);
        //let editor = self.editor.clone();
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
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            // Let the Editor handle backspace and delete natively
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            //.bg(themes::Theme.mantle)
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
                                                println!("sent message clicked!!!");
                                                println!("message is {}", this.get_text(cx));
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

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        //println!("left action is invoked");
        if self.selected_range.is_empty() {
            //println!("inside selected range empty");
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            //println!("inside selected range non-empty");
            self.move_to(self.selected_range.start, cx)
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx)
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    // Removed custom backspace and delete handlers; Editor will handle these actions.

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace("\n", " "), window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                (&self.content[self.selected_range.clone()]).to_string(),
            ));
        }
    }
    fn cut(&mut self, _: &Copy, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                (&self.content[self.selected_range.clone()]).to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        //println!("move_to offset: {}", offset);
        self.selected_range = offset..offset;
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: gpui::Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        let result = self
            .content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0);
        println!("previous_boundary result: {}, offset: {}", result, offset);
        return result;
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    pub fn reset(&mut self) {
        self.content = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
    }
}

impl EventEmitter<MessageEditorEvent> for MessageEditor {}

impl EntityInputHandler for MessageEditor {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.marked_range = Some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        assert_eq!(last_layout.text, self.content);
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

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
        let line_height = TextSize::Small.rems(cx).to_pixels(window.rem_size()) * 1.5;

        v_flex()
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_editor(window, cx))
    }
}
