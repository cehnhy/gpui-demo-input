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
    text_input_view: View<TextInput>,
    state_model: Model<State>,
    _update_state_model_task: Option<Task<()>>,
    list_state: ListState,
}

impl Root {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        // text input view
        let text_input_view = cx.new_view(|cx| {
            let text_input = TextInput::new(cx);
            text_input
        });
        cx.subscribe(&text_input_view, Self::on_input_event)
            .detach();

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
                    item.selected = true;
                }
                div().child(item).into_any_element()
            }
        });

        Self {
            text_input_view,
            state_model,
            _update_state_model_task: None,
            list_state,
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
                self._update_state_model_task = Some(cx.spawn(Self::do_update_state_model_task))
            }
            InputEvent::PressEnter => {
                // TODO open application
            }
            _ => {}
        };
    }

    async fn do_update_state_model_task(
        root_weak_view: WeakView<Self>,
        mut cx: AsyncWindowContext,
    ) {
        // do async
        root_weak_view
            .update(&mut cx, Self::update_state_model)
            .unwrap();
    }

    fn update_state_model(&mut self, cx: &mut ViewContext<Self>) {
        self.state_model.update(cx, |state, cx| {
            state.items.clear();
            state.selected_id = 0;

            let text_content = self.text_input_view.read(cx).text();
            if text_content.is_empty() {
                self.list_state.reset(0);
                cx.notify();
                return;
            }

            if let Ok(entries) = std::fs::read_dir("/Applications") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if file_name.ends_with(".app")
                            && file_name
                                .to_lowercase()
                                .contains(&text_content.to_lowercase())
                        {
                            state.items.push(ListItem::new(
                                false,
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

    fn up(&mut self, _: &Up, cx: &mut ViewContext<Self>) {
        self.state_model.update(cx, State::up);
        let selected_id = self.state_model.read(cx).selected_id;
        self.list_state.scroll_to_reveal_item(selected_id);
        cx.notify();
    }

    fn down(&mut self, _: &Down, cx: &mut ViewContext<Self>) {
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
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .size_full()
            .flex()
            .flex_col()
            .child(self.text_input_view.clone())
            .child(list(self.list_state.clone()).w_full().h_full())
    }
}

impl FocusableView for Root {
    fn focus_handle(&self, cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.text_input_view.focus_handle(cx)
    }
}

#[derive(Clone)]
struct State {
    selected_id: usize,
    items: Vec<ListItem>,
}

impl State {
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
    pub fn new(selected: bool, title: String, subtitle: String) -> Self {
        ListItem {
            selected,
            title: title.into(),
            subtitle: subtitle.into(),
        }
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
