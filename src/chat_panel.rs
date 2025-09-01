use crate::nrepl_client::NreplRequestResponse;
use crate::state::*;
use colors::*;
use gpui::*;

pub struct EvaluatedExprList {
    state: ListState,
}

impl Render for EvaluatedExprList {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .child(list(self.state.clone()).w_full().h_full())
    }
}

impl EvaluatedExprList {
    pub fn new(app: &mut App) -> Entity<Self> {
        app.new(|cx| {
            let state = cx.global::<StateModel>().inner.clone();
            cx.subscribe(&state, |this: &mut EvaluatedExprList, model, _event, cx| {
                let items = model.read(cx).items.clone();
                this.state = ListState::new(
                    items.len(),
                    ListAlignment::Bottom,
                    Pixels(20.),
                    move |idx, _win, _app| {
                        let item = items.get(idx).unwrap().clone();
                        div().child(item).into_any_element()
                    },
                );
                cx.notify();
            })
            .detach();

            EvaluatedExprList {
                state: ListState::new(0, ListAlignment::Bottom, Pixels(20.), move |_, _, _| {
                    div().into_any_element()
                }),
            }
        })
    }
}

/// The ChatPanel component, which displays a list of chat messages.
pub struct ChatPanel {
    pub messages: Entity<EvaluatedExprList>,
}

impl ChatPanel {
    pub fn new(app: &mut App) -> Entity<Self> {
        let list_view = EvaluatedExprList::new(app);
        app.new(|_| ChatPanel {
            messages: list_view,
        })
    }
}

impl Render for ChatPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Panel container
        let colors = Colors::default();
        let state = cx.global::<StateModel>();
        let items = state.inner.read(cx).items.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.background)
            .children(
                items
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
                                    .child(msg.req.clone()),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors.text)
                                    .child(msg.resp.clone()),
                            )
                    })
                    .collect::<Vec<_>>(),
            )
    }
}
