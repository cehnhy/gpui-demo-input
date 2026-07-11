use crate::app::query_parser::{DefaultQueryParser, QueryParser};
use chrono::{DateTime, Local};
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use prelude::FluentBuilder;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const CONTEXT: &str = "root";
const LIST_ITEM_ACTIVE_BG: u32 = 0x444444;
actions!(root, [Up, Down, ESC]);

fn format_time(time: DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

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
    state: Entity<Launcher>,
    list_state: ListState,
    time: SharedString,
    _update_time_task: Task<()>,
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
        let state = cx.new(|_cx| Launcher::new());

        // list state
        let list_state = ListState::new(0, ListAlignment::Top, px(20.));
        let time = format_time(Local::now()).into();

        Self {
            window_handle,
            query,
            state,
            list_state,
            time,
            _update_time_task: cx.spawn(Self::do_update_time_task),
            _update_list_task: Some(cx.spawn(Self::do_update_list_task)),
            _open_application_task: None,
        }
    }

    async fn do_update_time_task(self_weak_entity: WeakEntity<Self>, cx: &mut AsyncApp) {
        loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;

            if self_weak_entity
                .update(cx, |root, cx| {
                    root.time = format_time(Local::now()).into();
                    cx.notify();
                })
                .is_err()
            {
                break;
            }
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
            let text_content = self.query.read(cx).value();
            state.search(cx, &text_content);
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
        if state.launch() {
            cx.update_window(self.window_handle, |_, window, _| {
                window.remove_window();
            })
            .unwrap();
        }
    }

    fn open_item(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.state.update(cx, |state, _cx| {
            state.selected_id = idx;
        });
        self._open_application_task = Some(cx.spawn(Self::do_open_application_task));
    }

    fn select_last_item(&mut self, _: &Up, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, Launcher::up);
        let selected_id = self.state.read(cx).selected_id;
        self.list_state.scroll_to_reveal_item(selected_id);
        cx.notify();
    }

    fn select_next_item(&mut self, _: &Down, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, Launcher::down);
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
        let root_entity = cx.weak_entity();
        let mut h = 48;

        let mut state_len = self.state.read(cx).items.len();
        if state_len > 6 {
            state_len = 6;
        }
        if state_len > 0 {
            h += 8 + 59 * state_len
        }

        div()
            .id("root")
            .flex()
            .pt(px(310.0))
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
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .px_3()
                            .text_sm()
                            .text_color(rgb(0x929292))
                            .child(self.time.clone()),
                    )
                    .child(
                        div()
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
                                        cx.processor(move |root, idx, _window, app| {
                                            let state = root.state.read(app);
                                            let item: &ListItem = state.items.get(idx).unwrap();
                                            let mut item = item.clone();
                                            item.set_index(idx);
                                            if idx == state.selected_id {
                                                item.select();
                                            }
                                            let root_entity = root_entity.clone();

                                            div()
                                                .id(("list-item-click", idx))
                                                .on_click(move |_event, _window, cx| {
                                                    cx.stop_propagation();
                                                    root_entity
                                                        .update(cx, |root, cx| {
                                                            root.open_item(idx, cx)
                                                        })
                                                        .ok();
                                                })
                                                .child(item)
                                                .into_any_element()
                                        }),
                                    )
                                    .size_full(),
                                ),
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
struct Launcher {
    selected_id: usize,
    items: Vec<ListItem>,
}

impl Launcher {
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

    fn search(&mut self, _cx: &mut Context<Self>, query: &str) {
        self.reset();
        let query_parser = DefaultQueryParser::default();
        let query_items = query_parser.parse(query);
        for item in query_items {
            self.items.push(ListItem::new(
                item.title,
                item.subtitle,
                item.action,
                item.icon,
            ));
        }
    }

    fn launch(&self) -> bool {
        if let Some(item) = self.items.get(self.selected_id) {
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
            return true;
        }
        false
    }
}

#[derive(Clone, Debug, IntoElement)]
pub struct ListItem {
    index: usize,
    selected: bool,
    title: SharedString,
    subtitle: SharedString,
    action: SharedString,
    icon: PathBuf,
}

impl ListItem {
    pub fn new(title: String, subtitle: String, action: String, icon: PathBuf) -> Self {
        ListItem {
            index: 0,
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

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl RenderOnce for ListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id(("list-item", self.index))
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .when(self.selected, |this| this.bg(rgb(LIST_ITEM_ACTIVE_BG)))
            .hover(|this| this.bg(rgb(LIST_ITEM_ACTIVE_BG)))
            .my_0p5()
            .pl_1()
            .rounded_md()
            .text_color(rgb(0xCBCBCB))
            .text_xl()
            .child(img(self.icon).h(px(40.0)).w(px(40.0)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .when(self.subtitle.is_empty(), |this| {
                        this.child(div().child(self.title.clone()))
                            .line_height(px(55.0))
                    })
                    .when(!self.subtitle.is_empty(), |this| {
                        this.child(div().child(self.title.clone()))
                            .child(div().text_sm().child(self.subtitle.clone()))
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::format_time;
    use chrono::TimeZone;

    #[test]
    fn formats_time_for_the_launcher_label() {
        let time = chrono::Local
            .with_ymd_and_hms(2026, 7, 11, 14, 30, 25)
            .single()
            .expect("test time should be valid");

        assert_eq!(format_time(time), "2026-07-11 14:30:25");
    }
}
