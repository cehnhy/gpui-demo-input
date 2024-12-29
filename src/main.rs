mod input;

use gpui::*;
use input::Root;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        component::input::init(cx);
        component::theme::init(cx);

        let bounds = Bounds::centered(None, size(px(600.0), px(300.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |cx| {
                cx.new_view(|cx| {
                    let root = Root::new(cx);
                    cx.focus_self();
                    root
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
