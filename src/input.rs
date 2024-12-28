use component::input::{InputEvent, TextInput};
use gpui::*;

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
        let state_model = cx.new_model(|_cx| State { items: vec![] });
        cx.observe(&state_model, Self::on_state_model_notify)
            .detach();

        // list state
        let list_state = ListState::new(0, ListAlignment::Top, Pixels(20.), move |_, _| {
            div().into_any_element()
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
            InputEvent::Change(_text) => {}
            InputEvent::PressEnter => {
                self._update_state_model_task = Some(cx.spawn(Self::do_update_state_model_task))
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
            let t = self.text_input_view.read(cx).text();
            t.chars().for_each(|c| {
                state
                    .items
                    .push(ListItem::new(c.to_string(), c.to_string()))
            });
            cx.notify();
        });
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
    items: Vec<ListItem>,
}

#[derive(Clone, Debug, IntoElement)]
pub struct ListItem {
    title: SharedString,
    subtitle: SharedString,
}

impl ListItem {
    pub fn new(title: String, subtitle: String) -> Self {
        ListItem {
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
