mod input;

use gpui::*;
use input::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);
        input::init(cx);

        let bounds = Bounds::centered(None, size(px(600.0), px(295.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    let root = cx.new(|cx| {
                        let root = Root::new(window, cx);
                        cx.observe_window_activation(window, |_, window, _cx| {
                            if !window.is_window_active() {
                                window.remove_window();
                            }
                        })
                        .detach();
                        cx.focus_self(window);
                        root
                    });
                    cx.activate(true);
                    gpui_component::Root::new(root, window, cx)
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
