use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use prelude::FluentBuilder;
use std::{path::PathBuf, time::Instant};

const CONTEXT: &str = "root";
actions!(root, [Up, Down, ESC]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", Up, Some(CONTEXT)),
        KeyBinding::new("down", Down, Some(CONTEXT)),
        KeyBinding::new("escape", ESC, Some(CONTEXT)),
    ]);
}

pub struct Root {
    window_handle: AnyWindowHandle,
    query: Entity<InputState>,
    state: Entity<State>,
    list_state: ListState,
    _update_list_task: Option<Task<()>>,
    _open_application_task: Option<Task<()>>,
}

impl Root {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let window_handle = window.window_handle();
        // query
        let query = cx.new(|cx| InputState::new(window, cx));
        cx.subscribe(&query, Self::on_input_event).detach();

        // state
        let state = cx.new(|_cx| State::new());

        // list state
        let list_state = ListState::new(0, ListAlignment::Top, px(20.));

        Self {
            window_handle,
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
        let now = Instant::now();
        self.state.update(cx, |state, cx| {
            state.reset();

            #[cfg(target_os = "macos")]
            {
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
                                    PathBuf::from(""),
                                ));
                            }
                        }
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                let text_content = self.query.read(cx).value();

                let locales = get_languages_from_env();
                let entries = Iter::new(default_paths())
                    .entries(Some(&locales))
                    .collect::<Vec<_>>();
                for entry in entries {
                    if entry.no_display() {
                        continue;
                    }

                    let name = match entry.name(&locales) {
                        Some(name) => name.to_string(),
                        None => continue,
                    };

                    if !name.to_lowercase().contains(&text_content.to_lowercase()) {
                        continue;
                    }

                    let comment = match entry.comment(&locales) {
                        Some(comment) => comment.to_string(),
                        None => continue,
                    };

                    let action = match entry.exec() {
                        Some(exec) => exec.to_string(),
                        None => continue,
                    };

                    let mut icon = PathBuf::from("");
                    if let Some(icon_path) = entry.icon() {
                        if let Some(found_icon) = freedesktop_icons::lookup(&icon_path)
                            .with_cache()
                            .with_size(48)
                            .find()
                        {
                            icon = found_icon;
                        }
                    }

                    state.items.push(ListItem::new(name, comment, action, icon));
                }
            }

            self.list_state.reset(state.items.len());
            cx.notify();
        });
        println!("Update list took {:?}", now.elapsed());
    }

    // do async task
    async fn do_open_application_task(self_weak_entity: WeakEntity<Self>, cx: &mut AsyncApp) {
        self_weak_entity.update(cx, Self::open_application).unwrap();
    }
    fn open_application(&mut self, cx: &mut Context<Self>) {
        let state = self.state.read(cx);
        if let Some(item) = state.items.get(state.selected_id) {
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open")
                    .arg("-a")
                    .arg(item.subtitle.as_ref())
                    .output();
            }

            #[cfg(target_os = "linux")]
            {
                // Parse the Exec field to remove field codes (%f, %F, %u, %U, etc.)
                let exec_cmd = item.action.as_ref();
                let parts: Vec<&str> = exec_cmd.split_whitespace().collect();

                if let Some(command) = parts.first() {
                    let args: Vec<&str> = parts[1..]
                        .iter()
                        .filter(|arg| !arg.starts_with('%'))
                        .copied()
                        .collect();

                    let _ = std::process::Command::new(command).args(&args).spawn();
                }
            }

            cx.update_window(self.window_handle, |_, window, _| {
                window.remove_window();
            })
            .unwrap();
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

    fn cancel(&mut self, _: &ESC, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut h = 48;

        let mut state_len = self.state.read(cx).items.len();
        if state_len > 6 {
            state_len = 6;
        }
        if state_len > 0 {
            h += 8 + 59 * state_len
        }

        div()
            .flex()
            .pt(px(350.0))
            .items_start()
            .justify_center()
            .size_full()
            .on_mouse_down(gpui::MouseButton::Left, move |_event, window, _cx| {
                window.remove_window();
            })
            .child(
                div()
                    .key_context(CONTEXT)
                    .on_action(cx.listener(Self::select_last_item))
                    .on_action(cx.listener(Self::select_next_item))
                    .on_action(cx.listener(Self::cancel))
                    .w(px(800.0))
                    .h(px(h as f32))
                    .flex()
                    .flex_col()
                    .bg(rgba(0x1E1E1EFF))
                    .rounded(px(10.0))
                    .overflow_hidden()
                    .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div().p_2().child(
                            Input::new(&self.query)
                                .border_1()
                                .border_color(rgb(0x3a3a3a))
                                .bg(rgba(0x1E1E1EFF)),
                        ),
                    )
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
    action: SharedString,
    icon: PathBuf,
}

impl ListItem {
    pub fn new(title: String, subtitle: String, action: String, icon: PathBuf) -> Self {
        ListItem {
            selected: false,
            title: title.into(),
            subtitle: subtitle.into(),
            action: action.into(),
            icon,
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
            .items_center()
            .gap_1()
            .my_0p5()
            .pl_1()
            .rounded_md()
            // .hover(|s| s.bg(rgb(0x3a3a3a)))
            .text_color(rgb(0xCBCBCB))
            .text_xl()
            .child(img(self.icon).h(px(40.0)).w(px(40.0)))
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
