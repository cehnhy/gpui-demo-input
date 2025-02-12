mod input;

use gpui::*;
use input::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::input::init(cx);
        gpui_component::theme::init(cx);
        input::init(cx);

        cx.activate(true);

        let bounds = Bounds::centered(None, size(px(600.0), px(294.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Root::new(window, cx)),
        )
        .unwrap();
    });
}
