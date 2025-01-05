mod input;

use gpui::*;
use input::Root;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        component::input::init(cx);
        component::theme::init(cx);
        input::init(cx);

        cx.activate(true);

        let bounds = Bounds::centered(None, size(px(600.0), px(294.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |cx| cx.new_view(|cx| Root::new(cx)),
        )
        .unwrap();
    });
}
