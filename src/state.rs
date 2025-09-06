use gpui::*;

use crate::nrepl_client::*;

#[derive(Debug, Clone)]
pub struct NReplResult {
    pub value: SharedString,
    pub output: SharedString,
    pub error: SharedString,
    pub has_error: bool,
}

pub fn create_ui_eval_result(input: EvalResult) -> NReplResult {
    match input {
        EvalResult {
            value,
            output,
            error,
            has_error,
        } => {
            let new_value: SharedString = value.map(|x| x.into()).expect("nil".into());
            let new_output: SharedString = output.into();
            let new_error: SharedString = error.into();
            NReplResult {
                value: new_value,
                output: new_output,
                error: new_error,
                has_error: has_error,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct NreplRequest {
    pub id: usize,
    pub req: gpui::SharedString,
}

#[derive(Clone, Debug, IntoElement)]
pub struct NreplRequestResponse {
    pub id: usize,
    pub req: SharedString,
    pub resp: NReplResult,
}

impl RenderOnce for NreplRequestResponse {
    fn render(self, _: &mut Window, _app: &mut App) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .items_center()
            .py_2()
            .px_4()
            .border_t_1()
            //.border_color(themes.crust_light)
            //.hover(|s| s.bg(themes.base_blur))
            .text_xl()
            .children([self.req.clone(), self.resp.output.clone()])
    }
}

#[derive(Clone)]
pub struct State {
    pub count: usize, // hack for generating req-resp item ids
    pub items: Vec<NreplRequestResponse>,
}

#[derive(Clone)]
pub struct StateModel {
    pub inner: Entity<State>,
    pub client: Entity<NreplClient>,
}

impl StateModel {
    pub fn init(app: &mut App, port: u16) {
        let model = app.new(|_cx| State {
            count: 0,
            items: vec![],
        });

        let  client = app.new(|_cx| match NreplClient::connect("127.0.0.1", port) {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to connect: {}", e);
                panic!(
                    "Make sure nREPL server is running with: lein repl :headless :host 127.0.0.1 :port {}", port
                );
            }
        });

        let this = Self {
            inner: model,
            client: client,
        };
        app.set_global(this.clone());
    }

    pub fn update(f: impl FnOnce(&mut Self, &mut App), cx: &mut App) {
        if !cx.has_global::<Self>() {
            return;
        }
        cx.update_global::<Self, _>(|mut this, cx| {
            f(&mut this, cx);
        });
    }

    pub fn push(&self, item: NreplRequest, cx: &mut App) {
        self.inner.update(cx, |model, cx| {
            self.client.update(cx, |client, _| {
                let result = client.eval(item.req.trim());
                match result {
                    Ok(v) => {
                        let repl_entry = NreplRequestResponse {
                            id: item.id,
                            req: item.req,
                            resp: create_ui_eval_result(v.clone()),
                        };
                        println!("output: \n");
                        println!("{}", v.output);
                        model.items.push(repl_entry);
                    }
                    Err(e) => println!("error occured {}", e),
                }
            });
            model.count += 1;
            cx.emit(ListChangedEvent {});
        });
    }

    pub fn remove(&self, id: usize, cx: &mut App) {
        self.inner.update(cx, |model, cx| {
            let index = model.items.iter().position(|x| x.id == id).unwrap();
            model.items.remove(index);
            cx.emit(ListChangedEvent {});
        });
    }
}

impl Global for StateModel {}

#[derive(Clone, Debug)]
pub struct ListChangedEvent {}

impl EventEmitter<ListChangedEvent> for State {}

pub struct Nrepl {
    client: NreplClient,
}

impl Global for Nrepl {}

impl Clone for Nrepl {
    fn clone(&self) -> Self {
        let port = self.client.get_port();
        let mut client = match NreplClient::connect("127.0.0.1", port) {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to connect: {}", e);
                panic!(
                    "Make sure nREPL server is running with: lein repl :headless :host 127.0.0.1 :port {}",
                    port
                );
            }
        };
        Nrepl { client: client }
    }
}
impl Nrepl {
    pub fn init(app: &mut App) {
        let client = match NreplClient::connect("127.0.0.1", 64649) {
            Ok(c) => c,
            Err(e) => {
                println!("Failed to connect: {}", e);
                panic!(
                    "Make sure nREPL server is running with: lein repl :headless :host 127.0.0.1 :port 63067"
                );
            }
        };

        let nrepl = Nrepl { client: client };

        app.set_global(nrepl);
    }
}
