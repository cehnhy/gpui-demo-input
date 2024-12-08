use component::input::{InputEvent, TextInput};
use gpui::*;

pub struct Root {
    text_input: View<TextInput>,
    state_model: Model<State>,
    list_state: ListState,
    _task_update_list: Option<Task<()>>,
}

impl Root {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let text_input = cx.new_view(|cx| {
            let text_input = TextInput::new(cx);
            text_input
        });
        cx.subscribe(&text_input, Self::on_input_event).detach();

        let state_model = cx.new_model(|_cx| State { items: vec![] });
        cx.observe(&state_model, Self::on_state_model_notify)
            .detach();

        let list_state = ListState::new(0, ListAlignment::Top, Pixels(20.), move |_, _| {
            div().into_any_element()
        });

        Self {
            text_input,
            state_model,
            list_state,
            _task_update_list: None,
        }
    }

    fn on_input_event(
        &mut self,
        _text_input_view: View<TextInput>,
        input_event: &InputEvent,
        cx: &mut ViewContext<Self>,
    ) {
        match input_event {
            InputEvent::Change(_text) => {}
            InputEvent::PressEnter => {
                self._task_update_list = Some(cx.spawn(Self::update_state_model))
            }
            _ => {}
        };
    }

    async fn update_state_model(root_weak_view: WeakView<Self>, mut cx: AsyncWindowContext) {
        if let Some(root_view) = root_weak_view.upgrade() {
            root_view
                .update(&mut cx, |this, cx| {
                    this.state_model.update(cx, |state, cx| {
                        state.items.clear();
                        let t = this.text_input.read(cx).text();
                        t.chars().for_each(|c| {
                            state
                                .items
                                .push(ListItem::new(c.to_string(), c.to_string()))
                        });
                        cx.notify();
                    });
                })
                .unwrap();
        }
    }

    fn on_state_model_notify(&mut self, state_model: Model<State>, cx: &mut ViewContext<Self>) {
        let items = state_model.read(cx).items.clone();
        self.list_state = ListState::new(
            items.len(),
            ListAlignment::Top,
            Pixels(20.),
            move |idx, _cx| {
                let item = items.get(idx).unwrap().clone();
                div().child(item).into_any_element()
            },
        );
        cx.notify();
    }
}

impl Render for Root {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.text_input.clone())
            .child(list(self.list_state.clone()).w_full().h_full())
    }
}

impl FocusableView for Root {
    fn focus_handle(&self, cx: &gpui::AppContext) -> gpui::FocusHandle {
        self.text_input.focus_handle(cx)
    }
}

#[derive(Clone)]
struct State {
    items: Vec<ListItem>,
}

#[derive(Clone, Debug, IntoElement)]
pub struct ListItem {
    title: SharedString,
    subtitle: SharedString,
}

impl RenderOnce for ListItem {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x2a2a2a))
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

impl ListItem {
    pub fn new(title: String, subtitle: String) -> Self {
        ListItem {
            title: title.into(),
            subtitle: subtitle.into(),
        }
    }
}
