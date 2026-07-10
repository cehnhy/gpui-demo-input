pub mod input;
pub mod query_parser;
pub mod theme;

use crate::bridge::{event::Event, server::Server, Service as _};
use flume::{Receiver, Sender};
use futures::StreamExt;
use gpui::{layer_shell::*, *};
use gpui_platform::application;
use gpui_tokio::Tokio;
use tarpc::{server::Channel, tokio_serde::formats::Json};

pub fn run() {
    application().run(|cx: &mut App| {
        gpui_component::init(cx);
        input::init(cx);
        theme::setup_theme(cx);
        gpui_tokio::init(cx);
        cx.set_quit_mode(gpui::QuitMode::Explicit);

        let (tx, rx) = flume::unbounded::<Event>();
        listen(cx, tx);
        recv(cx, rx);
    });
}

fn listen(cx: &mut App, tx: Sender<Event>) {
    cx.spawn(async move |cx| {
        Tokio::spawn(cx, async move {
            let server = Server::new(tx);
            let socket_path = "/tmp/gpui-demo-input.sock";
            let _ = std::fs::remove_file(socket_path);

            let listener = tarpc::serde_transport::unix::listen(socket_path, Json::default)
                .await
                .unwrap();

            listener
                .filter_map(|r| futures::future::ready(r.ok()))
                .map(tarpc::server::BaseChannel::with_defaults)
                .map(|channel| {
                    let server = server.clone();
                    channel.execute(server.serve()).for_each(|f| async move {
                        tokio::spawn(f);
                    })
                })
                .buffer_unordered(10)
                .for_each(|_| async {})
                .await;
        })
        .await
        .unwrap();
    })
    .detach();
}

fn recv(cx: &mut App, rx: Receiver<Event>) {
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(event) = rx.recv_async().await {
            match event {
                Event::OpenLauncher {} => {
                    cx.update(|cx| {
                        open_launcher(cx);
                    });
                }
            }
        }
    })
    .detach();
}

fn open_launcher(cx: &mut App) {
    cx.open_window(
        WindowOptions {
            titlebar: None,
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "gpui-demo-input".to_string(),
                layer: layer_shell::Layer::Top,
                anchor: layer_shell::Anchor::TOP
                    | layer_shell::Anchor::RIGHT
                    | layer_shell::Anchor::BOTTOM
                    | layer_shell::Anchor::LEFT,
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
                    let root = input::Root::new(window, cx);
                    cx.focus_self(window);
                    root
                });
                cx.activate(true);
                gpui_component::Root::new(root, window, cx).bg(rgba(0x00000000))
            })
        },
    )
    .unwrap();

    cx.activate(true);
}
