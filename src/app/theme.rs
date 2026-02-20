use gpui::{hsla, App};
use gpui_component::Theme;

pub fn setup_theme(cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.colors.foreground = hsla(0.0, 0.0, 0.8, 1.0);
    theme.background = hsla(0.0, 0.0, 0.0, 0.0);
}
