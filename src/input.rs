use gpui::*;

pub struct Root {
    text_input: View<component::input::TextInput>,
    list_state: ListState,
    state_model: Model<State>,
    _task_update_list: Option<Task<()>>,
}

impl Root {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let text_input = cx.new_view(|cx| {
            let text_input = component::input::TextInput::new(cx);
            text_input
        });
        cx.subscribe(&text_input, Self::on_input_event).detach();

        let list_state = ListState::new(0, ListAlignment::Top, Pixels(20.), move |_, _| {
            div().into_any_element()
        });

        let state_model = cx.new_model(|_cx| State { items: vec![] });
        cx.observe(&state_model, |this, model, cx| {
            let items = model.read(cx).items.clone();
            this.list_state = ListState::new(
                items.len(),
                ListAlignment::Top,
                Pixels(20.),
                move |idx, _cx| {
                    let item = items.get(idx).unwrap().clone();
                    div().child(item).into_any_element()
                },
            );
            cx.notify();
        })
        .detach();

        Self {
            text_input,
            list_state,
            state_model,
            _task_update_list: None,
        }
    }

    fn on_input_event(
        &mut self,
        _: View<component::input::TextInput>,
        event: &component::input::InputEvent,
        cx: &mut ViewContext<Self>,
    ) {
        match event {
            component::input::InputEvent::Change(_text) => {}
            component::input::InputEvent::PressEnter => {
                self._task_update_list = Some(cx.spawn(Self::update_list))
            }
            _ => {}
        };
    }

    async fn update_list(view: WeakView<Self>, mut cx: AsyncWindowContext) {
        if let Some(view) = view.upgrade() {
            view.update(&mut cx, |this, cx| {
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
