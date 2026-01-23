mod input;
mod query_parser;

use gpui::{layer_shell::*, *};
use gpui_component::Theme;
use input::Root;

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);
        input::init(cx);

        let theme = Theme::global_mut(cx);
        theme.colors.foreground = hsla(0.0, 0.0, 0.79, 1.0);
        theme.background = hsla(0.0, 0.0, 0.0, 0.0);

        let bounds = Bounds::centered(None, size(px(1920.0), px(1080.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "gpui-demo-input".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::LEFT | Anchor::RIGHT | Anchor::BOTTOM | Anchor::TOP,
                    exclusive_zone: None,
                    exclusive_edge: None,
                    margin: Some((px(0.), px(0.), px(0.), px(0.))),
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
