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
        theme.colors.foreground = hsla(0.0, 0.0, 0.8, 1.0);
        theme.background = hsla(0.0, 0.0, 0.0, 0.0);

        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "gpui-demo-input".to_string(),
                    layer: Layer::Top,
                    anchor: Anchor::TOP | Anchor::RIGHT | Anchor::BOTTOM | Anchor::LEFT,
                    exclusive_zone: None,
                    exclusive_edge: None,
                    // due to func window_border() in gpui's Root component set client inset to 12px,
                    // and the window border size is 1px,
                    // and layer shell doesn't support server side decorations,
                    // so we need to set negative margin to make the window full screen.
                    margin: Some((px(-13.), px(11.), px(11.), px(-13.))), 
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
