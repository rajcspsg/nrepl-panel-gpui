use crate::message_editor::*;
use crate::thread::*;
use db::kvp::KEY_VALUE_STORE;
use dock::{DockPosition, PanelEvent};
use gpui::Subscription;

use feature_flags::*;
use gpui::*;
use icons::*;
use project::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings::*;
use std::ops::Not;
use std::sync::Arc;
use ui::*;
use workspace::*;

const AGENT_PANEL_KEY: &str = "agent_panel";

actions!(
    agent,
    [
        /// Creates a new text-based conversation thread.
        NewTextThread,
        /// Toggles the context picker interface for adding files, symbols, or other context.
        ToggleContextPicker,
        /// Toggles the menu to create new agent threads.
        ToggleNewThreadMenu,
        /// Toggles the navigation menu for switching between threads and views.
        ToggleNavigationMenu,
        /// Toggles the options menu for agent settings and preferences.
        ToggleOptionsMenu,
        /// Deletes the recently opened thread from history.
        DeleteRecentlyOpenThread,
        /// Toggles the profile selector for switching between agent profiles.
        ToggleProfileSelector,
        /// Removes all added context from the current conversation.
        RemoveAllContext,
        /// Expands the message editor to full size.
        ExpandMessageEditor,
        /// Opens the conversation history view.
        OpenHistory,
        /// Adds a context server to the configuration.
        AddContextServer,
        /// Removes the currently selected thread.
        RemoveSelectedThread,
        /// Starts a chat conversation with follow-up enabled.
        ChatWithFollow,
        /// Cycles to the next inline assist suggestion.
        CycleNextInlineAssist,
        /// Cycles to the previous inline assist suggestion.
        CyclePreviousInlineAssist,
        /// Moves focus up in the interface.
        FocusUp,
        /// Moves focus down in the interface.
        FocusDown,
        /// Moves focus left in the interface.
        FocusLeft,
        /// Moves focus right in the interface.
        FocusRight,
        /// Removes the currently focused context item.
        RemoveFocusedContext,
        /// Accepts the suggested context item.
        AcceptSuggestedContext,
        /// Opens the active thread as a markdown file.
        OpenActiveThreadAsMarkdown,
        /// Opens the agent diff view to review changes.
        OpenAgentDiff,
        /// Keeps the current suggestion or change.
        Keep,
        /// Rejects the current suggestion or change.
        Reject,
        /// Rejects all suggestions or changes.
        RejectAll,
        /// Keeps all suggestions or changes.
        KeepAll,
        /// Follows the agent's suggestions.
        Follow,
        /// Resets the trial upsell notification.
        ResetTrialUpsell,
        /// Resets the trial end upsell notification.
        ResetTrialEndUpsell,
        /// Continues the current thread.
        ContinueThread,
        /// Continues the thread with burn mode enabled.
        ContinueWithBurnMode,
        /// Toggles burn mode for faster responses.
        ToggleBurnMode,
    ]
);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    #[default]
    Zed,
    TextThread,
    NativeAgent,
}

#[derive(Serialize, Deserialize)]
struct SerializedAgentPanel {
    width: Option<Pixels>,
}

pub fn init(cx: &mut App) {
    cx.observe_new(
        |workspace: &mut Workspace, _window, _cx: &mut Context<Workspace>| {
            // Panel initialization logic can be added here
        },
    )
    .detach();
}

pub struct AgentPanel {
    fs: Arc<dyn Fs>,
    configuration: Option<Entity<AgentConfiguration>>,
    configuration_subscription: Option<Subscription>,
    new_thread_menu_handle: PopoverMenuHandle<ContextMenu>,
    agent_panel_menu_handle: PopoverMenuHandle<ContextMenu>,
    assistant_navigation_menu_handle: PopoverMenuHandle<ContextMenu>,
    assistant_navigation_menu: Option<Entity<ContextMenu>>,
    active_view: ActiveView,
    width: Option<Pixels>,
    height: Option<Pixels>,
    zoomed: bool,
    pending_serialization: Option<Task<Result<()>>>,
    selected_agent: AgentType,
}

pub struct AgentConfiguration {
    fs: Arc<dyn Fs>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    scrollbar_state: ScrollbarState,
}

impl AgentConfiguration {
    pub fn new(fs: Arc<dyn Fs>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let scroll_handle = ScrollHandle::new();
        let scrollbar_state = ScrollbarState::new(scroll_handle.clone());

        let mut this = Self {
            fs,
            focus_handle,
            scroll_handle,
            scrollbar_state,
        };
        this
    }
}

impl Focusable for AgentConfiguration {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
/* pub struct AgentPanelOnboarding {
    user_store: Entity<UserStore>,
    client: Arc<Client>,
    configured_providers: Vec<(IconName, SharedString)>,
    continue_with_zed_ai: Arc<dyn Fn(&mut Window, &mut App)>,
}*/

pub enum AssistantConfigurationEvent {
    NewThread,
}

impl EventEmitter<AssistantConfigurationEvent> for AgentConfiguration {}

impl AgentPanel {
    fn serialize(&mut self, cx: &mut Context<Self>) {
        let width = self.width;
        self.pending_serialization = Some(cx.background_spawn(async move {
            KEY_VALUE_STORE
                .write_kvp(
                    AGENT_PANEL_KEY.into(),
                    serde_json::to_string(&SerializedAgentPanel { width })?,
                )
                .await?;
            anyhow::Ok(())
        }));
    }

    pub fn set_selected_agent(&mut self, agent: AgentType, cx: &mut Context<Self>) {
        if self.selected_agent != agent {
            self.selected_agent = agent;
            self.serialize(cx);
        }
    }

    pub fn selected_agent(&self) -> AgentType {
        self.selected_agent
    }

    pub fn new(fs: Arc<dyn Fs>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create a placeholder active view - this would need proper initialization in a real implementation
        let active_view = ActiveView::Configuration;

        Self {
            fs,
            configuration: None,
            configuration_subscription: None,
            new_thread_menu_handle: PopoverMenuHandle::default(),
            agent_panel_menu_handle: PopoverMenuHandle::default(),
            assistant_navigation_menu_handle: PopoverMenuHandle::default(),
            assistant_navigation_menu: None,
            active_view,
            width: None,
            height: None,
            zoomed: false,
            pending_serialization: None,
            selected_agent: AgentType::default(),
        }
    }

    pub fn open_configuration(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let fs = self.fs.clone();

        //self.set_active_view(ActiveView::Configuration, window, cx);
        self.configuration = Some(cx.new(|cx| AgentConfiguration::new(fs, window, cx)));

        if let Some(configuration) = self.configuration.as_ref() {
            self.configuration_subscription = Some(cx.subscribe_in(
                configuration,
                window,
                Self::handle_agent_configuration_event,
            ));

            configuration.focus_handle(cx).focus(window);
        }
    }

    fn handle_agent_configuration_event(
        &mut self,
        _entity: &Entity<AgentConfiguration>,
        event: &AssistantConfigurationEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            AssistantConfigurationEvent::NewThread => {
                self.new_thread(window, cx);
                if let Some(thread) = self.active_thread(cx) {
                    thread.update(cx, |thread, cx| {
                        thread.set_configured_model(cx);
                    });
                }
            }
        }
    }

    fn new_thread(&mut self, window: &mut Window, cx: &mut Context<Self>) {}

    pub fn active_thread(&self, cx: &App) -> Option<Entity<Thread>> {
        match &self.active_view {
            _ => None,
        }
    }

    fn render_empty_state_section_header(
        &self,
        label: impl Into<SharedString>,
        action_slot: Option<AnyElement>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .mt_2()
            .pl_1p5()
            .pb_1()
            .w_full()
            .justify_between()
            .border_b_1()
            .border_color(cx.theme().colors().border_variant)
            .child(
                Label::new(label.into())
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .children(action_slot)
    }

    fn render_thread_empty_state(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        v_flex()
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .when(true, |this| {
                this.child(
                    v_flex()
                        .size_full()
                        .mx_auto()
                        .justify_center()
                        .items_center()
                        .gap_1()
                        .child(h_flex().child(Headline::new("Welcome to the Agent Panel")))
                        .when(true, |parent| {
                            parent
                                .child(h_flex().child(
                                    Label::new("Ask and build anything.").color(Color::Muted),
                                ))
                                .child(
                                    v_flex()
                                        .mt_2()
                                        .gap_1()
                                        .max_w_48()
                                        .child(
                                            Button::new("context", "Add Context")
                                                .label_size(LabelSize::Small)
                                                .icon(IconName::FileCode)
                                                .icon_position(IconPosition::Start)
                                                .icon_size(IconSize::Small)
                                                .icon_color(Color::Muted)
                                                .full_width(),
                                        )
                                        .child(
                                            Button::new("mode", "Switch Model")
                                                .label_size(LabelSize::Small)
                                                .icon(IconName::DatabaseZap)
                                                .icon_position(IconPosition::Start)
                                                .icon_size(IconSize::Small)
                                                .icon_color(Color::Muted)
                                                .full_width(),
                                        )
                                        .child(
                                            Button::new("settings", "View Settings")
                                                .label_size(LabelSize::Small)
                                                .icon(IconName::Settings)
                                                .icon_position(IconPosition::Start)
                                                .icon_size(IconSize::Small)
                                                .icon_color(Color::Muted)
                                                .full_width(),
                                        ),
                                )
                        }),
                )
            })
            .when(true, |parent| {
                let focus_handle = focus_handle.clone();
                parent
                    .overflow_hidden()
                    .p_1p5()
                    .justify_end()
                    .gap_1()
                    .child(
                        self.render_empty_state_section_header(
                            "Recent",
                            Some(
                                Button::new("view-history", "View All")
                                    .style(ButtonStyle::Subtle)
                                    .label_size(LabelSize::Small)
                                    .into_any_element(),
                            ),
                            cx,
                        ),
                    )
                    .child(v_flex().gap_1())
            })
    }

    fn render_toolbar_old(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        let active_thread1: Option<Thread> = match &self.active_view {
            _ => None,
        };

        let new_thread_menu = PopoverMenu::new("new_thread_menu")
            .trigger_with_tooltip(
                IconButton::new("new_thread_menu_btn", IconName::Plus).icon_size(IconSize::Small),
                Tooltip::text("New Thread…"),
            )
            .anchor(Corner::TopRight)
            .with_handle(self.new_thread_menu_handle.clone())
            .menu({
                let focus_handle = focus_handle.clone();
                move |window, cx| {
                    Some(ContextMenu::build(window, cx, |mut menu, _window, cx| {
                        menu = menu.context(focus_handle.clone());
                        menu
                    }))
                }
            });

        h_flex()
            .id("assistant-toolbar")
            .h(Tab::container_height(cx))
            .max_w_full()
            .flex_none()
            .justify_between()
            .gap_2()
            .bg(cx.theme().colors().tab_bar_background)
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .size_full()
                    .pl_1()
                    .gap_1()
                    .child(match &self.active_view {
                        ActiveView::Configuration => div()
                            .pl(DynamicSpacing::Base04.rems(cx))
                            .child(self.render_toolbar_back_button(cx))
                            .into_any_element(),
                        _ => self
                            .render_recent_entries_menu(IconName::MenuAlt, cx)
                            .into_any_element(),
                    })
                    .child(self.render_title_view(window, cx)),
            )
            .child(
                h_flex().h_full().gap_2().child(
                    h_flex()
                        .h_full()
                        .gap(DynamicSpacing::Base02.rems(cx))
                        .px(DynamicSpacing::Base08.rems(cx))
                        .border_l_1()
                        .border_color(cx.theme().colors().border)
                        .child(new_thread_menu)
                        .child(self.render_panel_options_menu(window, cx)),
                ),
            )
    }

    fn render_recent_entries_menu(
        &self,
        icon: IconName,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        PopoverMenu::new("agent-nav-menu")
            .trigger_with_tooltip(
                IconButton::new("agent-nav-menu", icon).icon_size(IconSize::Small),
                {
                    let focus_handle = focus_handle.clone();
                    move |window, cx| {
                        Tooltip::for_action_in(
                            "Toggle Panel Menu",
                            &ToggleNavigationMenu,
                            &focus_handle,
                            window,
                            cx,
                        )
                    }
                },
            )
            .anchor(Corner::TopLeft)
            .with_handle(self.assistant_navigation_menu_handle.clone())
            .menu({
                let menu = self.assistant_navigation_menu.clone();
                move |window, cx| {
                    if let Some(menu) = menu.as_ref() {
                        menu.update(cx, |_, cx| {
                            cx.defer_in(window, |menu, window, cx| {
                                menu.rebuild(window, cx);
                            });
                        })
                    }
                    menu.clone()
                }
            })
    }

    fn render_toolbar_back_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        IconButton::new("go-back", IconName::ArrowLeft)
            .icon_size(IconSize::Small)
            .tooltip({
                let focus_handle = focus_handle.clone();

                move |window, cx| {
                    Tooltip::for_action_in("Go Back", &workspace::GoBack, &focus_handle, window, cx)
                }
            })
    }

    fn render_panel_options_menu(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        let full_screen_label = if self.is_zoomed(window, cx) {
            "Disable Full Screen"
        } else {
            "Enable Full Screen"
        };

        PopoverMenu::new("agent-options-menu")
            .trigger_with_tooltip(
                IconButton::new("agent-options-menu", IconName::Ellipsis)
                    .icon_size(IconSize::Small),
                {
                    let focus_handle = focus_handle.clone();
                    move |window, cx| {
                        Tooltip::for_action_in(
                            "Toggle Agent Menu",
                            &ToggleOptionsMenu,
                            &focus_handle,
                            window,
                            cx,
                        )
                    }
                },
            )
            .anchor(Corner::TopRight)
            .with_handle(self.agent_panel_menu_handle.clone())
            .menu({
                let focus_handle = focus_handle.clone();
                move |window, cx| {
                    Some(ContextMenu::build(window, cx, |mut menu, _window, _| {
                        menu = menu.context(focus_handle.clone());

                        menu = menu
                            .header("MCP Servers")
                            .action(
                                "View Server Extensions",
                                Box::new(zed_actions::Extensions {
                                    category_filter: Some(
                                        zed_actions::ExtensionCategoryFilter::ContextServers,
                                    ),
                                    id: None,
                                }),
                            )
                            .action("Add Custom Server…", Box::new(AddContextServer))
                            .separator();

                        menu = menu
                            .separator()
                            .action(full_screen_label, Box::new(ToggleZoom));
                        menu
                    }))
                }
            })
    }

    fn render_title_view(&self, _window: &mut Window, cx: &Context<Self>) -> AnyElement {
        const LOADING_SUMMARY_PLACEHOLDER: &str = "Loading Summary…";

        let content = match &self.active_view {
            ActiveView::Thread { .. } => div().w_full().into_any_element(),

            ActiveView::Configuration => Label::new("Settings").truncate().into_any_element(),
        };

        h_flex()
            .key_context("TitleEditor")
            .id("TitleEditor")
            .flex_grow()
            .w_full()
            .max_w_full()
            .overflow_x_scroll()
            .child(content)
            .into_any()
    }

    fn render_toolbar_new(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(cx);

        let active_thread: Option<Thread> = match &self.active_view {
            _ => None,
        };

        let new_thread_menu = PopoverMenu::new("new_thread_menu")
            .trigger_with_tooltip(
                IconButton::new("new_thread_menu_btn", IconName::Plus).icon_size(IconSize::Small),
                {
                    let focus_handle = focus_handle.clone();
                    move |window, cx| {
                        Tooltip::for_action_in(
                            "New…",
                            &ToggleNewThreadMenu,
                            &focus_handle,
                            window,
                            cx,
                        )
                    }
                },
            )
            .anchor(Corner::TopLeft)
            .with_handle(self.new_thread_menu_handle.clone())
            .menu({
                let focus_handle = focus_handle.clone();

                move |window, cx| {
                    let active_thread = active_thread.clone();
                    Some(ContextMenu::build(window, cx, |mut menu, _window, cx| {
                        menu = menu
                            .context(focus_handle.clone())
                            .header("Zed Agent")
                            .item(
                                ContextMenuEntry::new("New Text Thread")
                                    .icon(IconName::NewTextThread)
                                    .icon_color(Color::Muted)
                                    .action(NewTextThread.boxed_clone()),
                            )
                            .separator()
                            .header("External Agents");

                        menu
                    }))
                }
            });

        h_flex()
            .id("agent-panel-toolbar")
            .h(Tab::container_height(cx))
            .max_w_full()
            .flex_none()
            .justify_between()
            .gap_2()
            .bg(cx.theme().colors().tab_bar_background)
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .size_full()
                    .gap(DynamicSpacing::Base08.rems(cx))
                    .child(match &self.active_view {
                        ActiveView::Configuration => {
                            div().pl(DynamicSpacing::Base04.rems(cx)).into_any_element()
                        }
                        _ => h_flex()
                            .h_full()
                            .px(DynamicSpacing::Base04.rems(cx))
                            .border_r_1()
                            .border_color(cx.theme().colors().border)
                            .child(h_flex().px_0p5().gap_1p5())
                            .into_any_element(),
                    }),
            )
            .child(
                h_flex().h_full().gap_2().child(
                    h_flex()
                        .h_full()
                        .gap(DynamicSpacing::Base02.rems(cx))
                        .pl(DynamicSpacing::Base04.rems(cx))
                        .pr(DynamicSpacing::Base06.rems(cx))
                        .border_l_1()
                        .border_color(cx.theme().colors().border)
                        .child(new_thread_menu), //.child(self.render_recent_entries_menu(IconName::HistoryRerun, cx))
                                                 //.child(self.render_panel_options_menu(window, cx)),
                ),
            )
    }

    fn render_toolbar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if cx.has_flag::<feature_flags::AcpFeatureFlag>() {
            self.render_toolbar_new(window, cx).into_any_element()
        } else {
            self.render_toolbar_old(window, cx).into_any_element()
        }
    }
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgentDockPosition {
    Left,
    #[default]
    Right,
    Bottom,
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DefaultView {
    #[default]
    Thread,
    TextThread,
}

#[derive(Default, Clone, Debug)]
pub struct AgentSettings {
    pub enabled: bool,
    pub button: bool,
    pub dock: AgentDockPosition,
    pub default_width: Pixels,
    pub default_height: Pixels,
}

impl Global for AgentSettings {}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentProfileId(pub Arc<str>);

impl AgentProfileId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for AgentProfileId {
    fn default() -> Self {
        Self("write".into())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, JsonSchema)]
pub struct AgentSettingsContent {
    /// Whether the Agent is enabled.
    ///
    /// Default: true
    enabled: Option<bool>,
    /// Whether to show the agent panel button in the status bar.
    ///
    /// Default: true
    button: Option<bool>,
    /// Where to dock the agent panel.
    ///
    /// Default: right
    dock: Option<AgentDockPosition>,
    /// Default width in pixels when the agent panel is docked to the left or right.
    ///
    /// Default: 640
    default_width: Option<f32>,
    /// Default height in pixels when the agent panel is docked to the bottom.
    ///
    /// Default: 320
    default_height: Option<f32>,

    /// The default profile to use in the Agent.
    ///
    /// Default: write
    default_profile: Option<AgentProfileId>,
    /// Which view type to show by default in the agent panel.
    ///
    /// Default: "thread"
    default_view: Option<DefaultView>,
    /// Whenever a tool action would normally wait for your confirmation
    /// that you allow it, always choose to allow it.
    ///
    /// Default: false
    always_allow_tool_actions: Option<bool>,

    play_sound_when_agent_done: Option<bool>,
    /// Whether to stream edits from the agent as they are received.
    ///
    /// Default: false
    stream_edits: Option<bool>,
    /// Whether to display agent edits in single-file editors in addition to the review multibuffer pane.
    ///
    /// Default: true
    single_file_review: Option<bool>,

    /// Whether to show thumb buttons for feedback in the agent panel.
    ///
    /// Default: true
    enable_feedback: Option<bool>,
    /// Whether to have edit cards in the agent panel expanded, showing a preview of the full diff.
    ///
    /// Default: true
    expand_edit_card: Option<bool>,
    /// Whether to have terminal cards in the agent panel expanded, showing the whole command output.
    ///
    /// Default: true
    expand_terminal_card: Option<bool>,
    /// Whether to always use cmd-enter (or ctrl-enter on Linux) to send messages in the agent panel.
    ///
    /// Default: false
    use_modifier_to_send: Option<bool>,
}

impl AgentSettingsContent {
    pub fn set_dock(&mut self, dock: AgentDockPosition) {
        self.dock = Some(dock);
    }
}

impl Settings for AgentSettings {
    const KEY: Option<&'static str> = Some("agent");

    const FALLBACK_KEY: Option<&'static str> = Some("assistant");

    const PRESERVED_KEYS: Option<&'static [&'static str]> = Some(&["version"]);

    type FileContent = AgentSettingsContent;

    fn load(
        sources: SettingsSources<Self::FileContent>,
        _: &mut gpui::App,
    ) -> anyhow::Result<Self> {
        let mut settings = AgentSettings::default();

        for value in sources.defaults_and_customizations() {
            merge(&mut settings.enabled, value.enabled);
            merge(&mut settings.button, value.button);
            merge(&mut settings.dock, value.dock);
            merge(
                &mut settings.default_width,
                value.default_width.map(Into::into),
            );
            merge(
                &mut settings.default_height,
                value.default_height.map(Into::into),
            );
        }

        Ok(settings)
    }

    fn import_from_vscode(vscode: &settings::VsCodeSettings, current: &mut Self::FileContent) {
        if let Some(b) = vscode
            .read_value("chat.agent.enabled")
            .and_then(|b| b.as_bool())
        {
            current.enabled = Some(b);
            current.button = Some(b);
        }
    }
}

fn agent_panel_dock_position(cx: &App) -> DockPosition {
    match AgentSettings::get_global(cx).dock {
        AgentDockPosition::Left => DockPosition::Left,
        AgentDockPosition::Bottom => DockPosition::Bottom,
        AgentDockPosition::Right => DockPosition::Right,
    }
}

impl EventEmitter<PanelEvent> for AgentPanel {}

pub enum ActiveView {
    Thread {
        message_editor: Entity<MessageEditor>,
        change_title_editor: Entity<MessageEditor>,
        _subscriptions: Vec<gpui::Subscription>,
    },
    Configuration,
}

impl Focusable for AgentPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match &self.active_view {
            ActiveView::Thread { message_editor, .. } => message_editor.focus_handle(cx),
            ActiveView::Configuration => {
                if let Some(configuration) = self.configuration.as_ref() {
                    configuration.focus_handle(cx)
                } else {
                    cx.focus_handle()
                }
            }
        }
    }
}

impl Panel for AgentPanel {
    fn persistent_name() -> &'static str {
        "AgentPanel"
    }

    fn position(&self, _window: &Window, cx: &App) -> DockPosition {
        agent_panel_dock_position(cx)
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        position != DockPosition::Bottom
    }

    fn set_position(&mut self, position: DockPosition, _: &mut Window, cx: &mut Context<Self>) {
        settings::update_settings_file::<AgentSettings>(self.fs.clone(), cx, move |settings, _| {
            let dock = match position {
                DockPosition::Left => AgentDockPosition::Left,
                DockPosition::Bottom => AgentDockPosition::Bottom,
                DockPosition::Right => AgentDockPosition::Right,
            };
            settings.set_dock(dock);
        });
    }

    fn size(&self, window: &Window, cx: &App) -> Pixels {
        let settings = AgentSettings::get_global(cx);
        match self.position(window, cx) {
            DockPosition::Left | DockPosition::Right => {
                self.width.unwrap_or(settings.default_width)
            }
            DockPosition::Bottom => self.height.unwrap_or(settings.default_height),
        }
    }

    fn set_size(&mut self, size: Option<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        match self.position(window, cx) {
            DockPosition::Left | DockPosition::Right => self.width = size,
            DockPosition::Bottom => self.height = size,
        }
        self.serialize(cx);
        cx.notify();
    }

    fn set_active(&mut self, _active: bool, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn remote_id() -> Option<proto::PanelId> {
        Some(proto::PanelId::AssistantPanel)
    }

    fn icon(&self, _window: &Window, cx: &App) -> Option<IconName> {
        (self.enabled(cx) && AgentSettings::get_global(cx).button).then_some(IconName::ZedAssistant)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Agent Panel")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        Box::new(zed_actions::ToggleFocus)
    }

    fn activation_priority(&self) -> u32 {
        3
    }

    fn enabled(&self, cx: &App) -> bool {
        client::DisableAiSettings::get_global(cx).disable_ai.not()
            && AgentSettings::get_global(cx).enabled
    }

    fn is_zoomed(&self, _window: &Window, _cx: &App) -> bool {
        self.zoomed
    }

    fn set_zoomed(&mut self, zoomed: bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.zoomed = zoomed;
        cx.notify();
    }
}

impl Render for AgentPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("agent-panel")
            .size_full()
            .child(div().p_4().child("Agent Panel Content"))
            .child(self.render_toolbar(_window, cx))
    }
}

fn merge<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}
