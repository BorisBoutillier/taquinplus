use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TaquinPlus".to_string(),
                resolution: [800.0, 600.0].into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Component, Debug)]
struct Id(i32, i32);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let projection = OrthographicProjection {
        far: 1000.,
        near: -1000.,
        ..default()
    };
    let mut camera_tf = Transform::from_xyz(0.0, 0.0, 20.0);
    camera_tf.look_at(Vec3::ZERO, Vec3::Y);
    commands.spawn(Camera3dBundle {
        projection: Projection::Orthographic(projection),
        transform: camera_tf,
        ..default()
    });
    let image_handle = asset_server.load("images/1.png");
    for x in -2..=2 {
        for y in -2..=2 {
            let material = if (x, y) == (2, -2) {
                StandardMaterial {
                    base_color_texture: Some(image_handle.clone()),
                    ..default()
                }
            } else {
                Color::GREEN.into()
            };
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Box::new(90.0, 90.0, 5.))),
                    material: materials.add(material),
                    transform: Transform::from_xyz(x as f32 * 100.0, y as f32 * 100.0, 0.0),
                    ..default()
                })
                .insert(Id(x, y));
        }
    }
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            ..default()
        },
        ..default()
    });
}

fn update(time: Res<Time>, mut query: Query<(&mut Transform, &Id)>) {
    for (mut tf, id) in query.iter_mut() {
        if id.0 == -2 && id.1 == 2 {
            tf.rotate(Quat::from_axis_angle(Vec3::Y, time.delta_seconds()))
        }
        if id.0 == 2 && id.1 == -2 {
            tf.rotate(Quat::from_axis_angle(Vec3::X, time.delta_seconds()))
        }
    }
}
