mod input;

use gpui::{layer_shell::*, *};
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
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "gpui".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::LEFT | Anchor::RIGHT | Anchor::BOTTOM | Anchor::TOP,
                    exclusive_zone: None,
                    exclusive_edge: None,
                    margin: Some((px(10.), px(10.), px(10.), px(10.))),
                    keyboard_interactivity: KeyboardInteractivity::OnDemand,
                }),
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    let root = cx.new(|cx| {
                        let root = Root::new(window, cx);
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
