use bevy::prelude::*;
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputSubmitEvent};
use bevy_wry_webview::{
    ipc::{FetchEvent, IpcQueue, IpcSender},
    UiWebViewBundle, WebViewHandle, WebViewLocation, WebViewMarker, WebViewPlugin,
};
use serde::Deserialize;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebViewPlugin)
        .add_plugins(TextInputPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (moving_webview, log_msgs, text_listener))
        .run();
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum Msg {
    Count { name: String, count: u16 },
    OtherCount { name: String, count: u16 },
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                border: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(5.0)),
                align_self: AlignSelf::FlexEnd,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: Color::RED.into(),
            ..default()
        },
        TextInput {
            text_style: TextStyle {
                font_size: 40.,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn(UiWebViewBundle::<String, Msg> {
        node_bundle: NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(400.0),
                height: Val::Px(400.0),
                ..Default::default()
            },
            ..Default::default()
        },
        location: WebViewLocation::Html(
            r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <script>
var clickCount = 0;
addEventListener("click", (event) => {window.sendMessage({ type: clickCount % 2 == 0 ? 'Count' : 'OtherCount', name: 'cube', count: clickCount++ })});
window.processMessage = (item) => { document.getElementById('inner-ele').innerText = item; }
        </script>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
    </head>
    <body>
        <div id="ele">
            <div id="inner-ele">Positioned element on transparent background</div>
        </div>
    </body>
    <style>
html, body {
    width: 100vw;
    height: 100vh;
    background-color: rgba(255, 192, 203, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
}

#ele {
    background-color: blue;
    color: white;
    width: 50%;
    height: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
}
    </style>
</html>
                      "#
            .to_owned(),
        ),
        ..Default::default()
    });

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn moving_webview(time: Res<Time>, mut query: Query<&mut Style, With<WebViewMarker>>) {
    let _ = query.get_single_mut().map(|mut style| {
        let top = Val::Px(((time.elapsed_seconds().sin() / 2.0) + 0.5) * 500.0);
        let left = Val::Px(((time.elapsed_seconds().cos() / 2.0) + 0.5) * 500.0);

        *style = Style {
            top,
            left,
            ..style.clone()
        };
    });
}

fn log_msgs(mut query: Query<&mut IpcQueue<Msg>>) {
    let mut ipc = query.single_mut();
    for i in &mut ipc {
        println!("{:?}", i);
    }
}

fn text_listener(
    mut events: EventReader<TextInputSubmitEvent>,
    mut writer: EventWriter<FetchEvent>,
    query: Query<(&WebViewHandle, &mut IpcSender<String>)>,
) {
    if let Ok((wvhandle, ipc_handler)) = query.get_single() {
        for event in events.read() {
            writer.send(ipc_handler.send(*wvhandle, event.value.clone()));
        }
    }
}
