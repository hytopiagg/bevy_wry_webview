use bevy::prelude::*;
use bevy_wry_webview::{
    ipc::{SendBytes, WryMessage},
    UiWebViewBundle, WebViewDespawning, WebViewHandle, WebViewLocation, WebViewMarker,
    WebViewPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WebViewPlugin)
        .insert_resource(ButtonClickCount(0))
        .add_systems(Startup, setup)
        //.add_systems(Update, moving_webview)
        .add_systems(Update, button_system)
        .add_systems(Update, handle_message)
        .run();
}

#[derive(Component)]
struct ColoredCube;

#[derive(Component)]
struct CountView;

#[derive(Resource)]
struct ButtonClickCount(u8);

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(5.0)),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
                border_color: BorderColor(Color::BLACK),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            });
        });

    commands.spawn(UiWebViewBundle {
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
        location: WebViewLocation::Html(include_str!("./cube.html").to_owned()),
        ..Default::default()
    });

    commands.spawn((
        UiWebViewBundle {
            node_bundle: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(400.0),
                    height: Val::Px(400.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            location: WebViewLocation::Html(include_str!("./cube1.html").to_owned()),
            ..Default::default()
        },
        CountView,
    ));

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        ColoredCube,
    ));
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

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    query: Query<&WebViewHandle, With<CountView>>,
    mut writer: EventWriter<SendBytes>,
    mut counter: ResMut<ButtonClickCount>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                (*counter).0 += 1;
                border_color.0 = Color::RED;
                writer.send(SendBytes(query.single().clone(), vec![(*counter).0]));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn handle_message(
    mut reader: EventReader<WryMessage>,
    mut bg_color: Query<&mut Handle<StandardMaterial>, With<ColoredCube>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut color: Option<Color> = None;
    for WryMessage(_, data) in &mut reader {
        color = Some(Color::rgb(
            data[0] as f32 / 255.0,
            data[1] as f32 / 255.0,
            data[2] as f32 / 255.0,
        ))
    }
    if let Some(color) = color {
        *bg_color.single_mut() = materials.add(color.into());
    }
}
