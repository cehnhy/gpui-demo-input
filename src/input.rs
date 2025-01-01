use component::input::{InputEvent, TextInput};
use gpui::*;
use prelude::FluentBuilder;

const CONTEXT: &str = "root";
actions!(root, [Up, Down]);

pub fn init(cx: &mut AppContext) {
    cx.bind_keys([
        KeyBinding::new("up", Up, Some(CONTEXT)),
        KeyBinding::new("down", Down, Some(CONTEXT)),
    ]);
}

pub struct Root {
    query_view: View<TextInput>,
    state_model: Model<State>,
    list_state: ListState,
    _update_list_task: Option<Task<()>>,
    _open_application_task: Option<Task<()>>,
}

impl Root {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        // query view
        let query_view = cx.new_view(|cx| {
            let test_input = TextInput::new(cx);
            test_input
        });
        cx.subscribe(&query_view, Self::on_input_event).detach();

        // state model
        let state_model = cx.new_model(|_cx| State {
            selected_id: 0,
            items: vec![],
        });

        // list state
        let list_state = ListState::new(0, ListAlignment::Top, Pixels(20.), {
            let state_model = state_model.clone();
            move |idx, cx| {
                let state = state_model.read(cx);
                let mut item = state.items.get(idx).unwrap().clone();
                if idx == state.selected_id {
                    item.select();
                }
                div().child(item).into_any_element()
            }
        });

        Self {
            query_view,
            state_model,
            list_state,
            _update_list_task: Some(cx.spawn(Self::do_update_list_task)),
            _open_application_task: None,
        }
    }

    fn on_input_event(
        &mut self,
        _text_input_view: View<TextInput>,
        input_event: &InputEvent,
        cx: &mut ViewContext<Self>,
    ) {
        match input_event {
            InputEvent::Change(_text) => {
                self._update_list_task = Some(cx.spawn(Self::do_update_list_task))
            }
            InputEvent::PressEnter => {
                self._open_application_task = Some(cx.spawn(Self::do_open_application_task))
            }
            _ => {}
        };
    }

    // do async
    async fn do_update_list_task(root_weak_view: WeakView<Self>, mut cx: AsyncWindowContext) {
        root_weak_view.update(&mut cx, Self::update_list).unwrap();
    }

    fn update_list(&mut self, cx: &mut ViewContext<Self>) {
        self.state_model.update(cx, |state, cx| {
            state.reset();

            if let Ok(entries) = std::fs::read_dir("/Applications") {
                let text_content = self.query_view.read(cx).text();
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

    // do async
    async fn do_open_application_task(root_weak_view: WeakView<Self>, mut cx: AsyncWindowContext) {
        root_weak_view
            .update(&mut cx, Self::open_application)
            .unwrap();
    }

    fn open_application(&mut self, cx: &mut ViewContext<Self>) {
        let state = self.state_model.read(cx);
        if let Some(item) = state.items.get(state.selected_id) {
            let _ = std::process::Command::new("open")
                .arg("-a")
                .arg(item.subtitle.as_ref())
                .output();
        }
    }

    fn select_last(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        self.state_model.update(cx, State::up);
        let selected_id = self.state_model.read(cx).selected_id;
        self.list_state.scroll_to_reveal_item(selected_id);
        cx.notify();
    }

    fn select_next(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
        self.state_model.update(cx, State::down);
        let selected_id = self.state_model.read(cx).selected_id;
        self.list_state.scroll_to_reveal_item(selected_id);
        cx.notify();
    }
}

impl Render for Root {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(Self::select_next))
            .size_full()
            .flex()
            .flex_col()
            .child(self.query_view.clone())
            .child(list(self.list_state.clone()).w_full().h_full())
    }
}

impl FocusableView for Root {
    fn focus_handle(&self, cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.query_view.focus_handle(cx)
    }
}

#[derive(Clone)]
struct State {
    selected_id: usize,
    items: Vec<ListItem>,
}

impl State {
    fn reset(&mut self) {
        self.selected_id = 0;
        self.items.clear();
    }

    fn up(&mut self, _cx: &mut ModelContext<Self>) {
        if self.items.len() == 0 {
            return;
        }
        if self.selected_id != 0 {
            self.selected_id -= 1;
        } else {
            self.selected_id = self.items.len() - 1;
        }
    }

    fn down(&mut self, _cx: &mut ModelContext<Self>) {
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
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .when(self.selected, |this| this.bg(rgb(0x2a2a2a)))
            .items_start()
            .p_2()
            .m_2()
            .rounded_md()
            .hover(|s| s.bg(rgb(0x3a3a3a)))
            .text_color(rgb(0xffffff))
            .text_xl()
            .child(self.title.clone())
            .child(div().flex().text_sm().child(self.subtitle.clone()))
    }
}
