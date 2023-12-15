use bevy::prelude::*;
use bevy_wry_webview::{UiWebViewBundle, WebViewLocation, WebViewPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebViewPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiWebViewBundle {
        node_bundle: NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        location: WebViewLocation::Url("https://google.com".to_owned()),
        ..Default::default()
    });
}
