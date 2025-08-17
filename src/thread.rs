use gpui::*;
use project::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, JsonSchema,
)]
pub struct ThreadId(Arc<str>);

#[derive(Clone)]
pub struct Thread {
    pub id: ThreadId,
}

impl Thread {
    pub fn new(id: &str) -> Thread {
        let id = ThreadId(Arc::from(id));
        Thread { id }
    }
    pub fn set_configured_model(&mut self, cx: &mut Context<Self>) {}
}

#[derive(Default, Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = agent)]
#[serde(deny_unknown_fields)]
pub struct NewThread {
    #[serde(default)]
    pub from_thread_id: Option<ThreadId>,
}
