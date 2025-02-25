mod input;

use gpui::*;
use input::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::input::init(cx);
        gpui_component::theme::init(cx);
        input::init(cx);

        let bounds = Bounds::centered(None, size(px(600.0), px(318.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    let root = Root::new(window, cx);
                    cx.focus_self(window);
                    root
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
