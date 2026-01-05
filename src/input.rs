use gpui::*;
use gpui_component::input::{InputEvent, InputState, TextInput};
use prelude::FluentBuilder;
use std::path::PathBuf;

const CONTEXT: &str = "root";
actions!(root, [Up, Down]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", Up, Some(CONTEXT)),
        KeyBinding::new("down", Down, Some(CONTEXT)),
    ]);
}

pub struct Root {
    query: Entity<InputState>,
    state: Entity<State>,
    list_state: ListState,
    _update_list_task: Option<Task<()>>,
    _open_application_task: Option<Task<()>>,
}

impl Root {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // query
        let query = cx.new(|cx| InputState::new(window, cx));
        cx.subscribe(&query, Self::on_input_event).detach();

        // state
        let state = cx.new(|_cx| State::new());

        // list state
        let list_state = ListState::new(0, ListAlignment::Top, Pixels(20.));

        Self {
            query,
            state,
            list_state,
            _update_list_task: Some(cx.spawn(Self::do_update_list_task)),
            _open_application_task: None,
        }
    }

    // on input event
    fn on_input_event(
        &mut self,
        _query: Entity<InputState>,
        input_event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        match input_event {
            InputEvent::Change => {
                self._update_list_task = Some(cx.spawn(Self::do_update_list_task))
            }
            InputEvent::PressEnter { .. } => {
                self._open_application_task = Some(cx.spawn(Self::do_open_application_task))
            }
            _ => {}
        };
    }

    // do async task
    async fn do_update_list_task(self_weak_entity: WeakEntity<Self>, cx: &mut AsyncApp) {
        self_weak_entity.update(cx, Self::update_list).unwrap();
    }
    fn update_list(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.reset();

            if let Ok(entries) = std::fs::read_dir("/Applications") {
                let text_content = self.query.read(cx).value();
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if file_name.ends_with(".app")
                            && (file_name
                                .to_lowercase()
                                .contains(&text_content.to_lowercase())
                                || text_content.is_empty())
                        {
                            state.items.push(ListItem::new(
                                file_name.to_string(),
                                path.display().to_string(),
                            ));
                        }
                    }
                }
            }

            self.list_state.reset(state.items.len());
            cx.notify();
        });
    }

    // do async task
    async fn do_open_application_task(self_weak_entity: WeakEntity<Self>, cx: &mut AsyncApp) {
        self_weak_entity.update(cx, Self::open_application).unwrap();
    }
    fn open_application(&mut self, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        if let Some(item) = state.items.get(state.selected_id) {
            let _ = std::process::Command::new("open")
                .arg("-a")
                .arg(item.subtitle.as_ref())
                .output();
        }
    }

    fn select_last_item(&mut self, _: &Up, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, State::up);
        let selected_id = self.state.read(cx).selected_id;
        self.list_state.scroll_to_reveal_item(selected_id);
        cx.notify();
    }

    fn select_next_item(&mut self, _: &Down, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, State::down);
        let selected_id = self.state.read(cx).selected_id;
        self.list_state.scroll_to_reveal_item(selected_id);
        cx.notify();
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::select_last_item))
            .on_action(cx.listener(Self::select_next_item))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x000000))
            .child(div().p_2().child(TextInput::new(&self.query)))
            .child(
                div().flex_1().pb_2().px_2().child(
                    list(
                        self.list_state.clone(),
                        cx.processor(|root, idx, _window, app| {
                            let state = root.state.read(app);
                            let item: &ListItem = state.items.get(idx).unwrap();
                            let mut item = item.clone();
                            if idx == state.selected_id {
                                item.select();
                            }
                            div().child(item).into_any_element()
                        }),
                    )
                    .size_full(),
                ),
            )
    }
}

impl Focusable for Root {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.query.focus_handle(cx)
    }
}

#[derive(Clone)]
struct State {
    selected_id: usize,
    items: Vec<ListItem>,
}

impl State {
    fn new() -> Self {
        Self {
            selected_id: 0,
            items: vec![],
        }
    }

    fn reset(&mut self) {
        self.selected_id = 0;
        self.items.clear();
    }

    fn up(&mut self, _cx: &mut Context<Self>) {
        if self.items.len() == 0 {
            return;
        }
        if self.selected_id != 0 {
            self.selected_id -= 1;
        } else {
            self.selected_id = self.items.len() - 1;
        }
    }

    fn down(&mut self, _cx: &mut Context<Self>) {
        if self.items.len() == 0 {
            return;
        }
        if self.selected_id != self.items.len() - 1 {
            self.selected_id += 1;
        } else {
            self.selected_id = 0;
        }
    }
}

#[derive(Clone, Debug, IntoElement)]
pub struct ListItem {
    selected: bool,
    title: SharedString,
    subtitle: SharedString,
}

impl ListItem {
    pub fn new(title: String, subtitle: String) -> Self {
        ListItem {
            selected: false,
            title: title.into(),
            subtitle: subtitle.into(),
        }
    }

    pub fn select(&mut self) {
        self.selected = true;
    }
}

impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .when(self.selected, |this| this.bg(rgb(0x2a2a2a)))
            .items_start()
            .my_0p5()
            .rounded_md()
            .hover(|s| s.bg(rgb(0x3a3a3a)))
            .text_color(rgb(0xffffff))
            .text_xl()
            .child(
                img(PathBuf::from(
                    "./app-icon.png", // TODO
                ))
                .h(px(56.0))
                .w(px(56.0)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(self.title.clone())
                    .child(div().flex().text_sm().child(self.subtitle.clone())),
            )
    }
}
