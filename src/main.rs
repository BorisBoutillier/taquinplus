use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .insert_resource(ClearColor(Color::BLACK))
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
    commands.spawn(Camera3dBundle {
        projection: Projection::Orthographic(projection),
        transform: Transform::from_xyz(0.0, 0., 20.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    let image_handle = asset_server.load("images/1.png");
    for x in -2..=2 {
        for y in -2..=2 {
            let uv_x1 = 0.2 * (x + 2) as f32;
            let uv_x2 = uv_x1 + 0.2;
            let uv_y1 = 0.2 * (4 - (y + 2)) as f32;
            let uv_y2 = uv_y1 + 0.2;
            let mut mesh = Mesh::from(shape::Cube::new(1.));
            #[rustfmt::skip]
            let uvs = vec![
                // Assigning the UV coords for the top side.
                [uv_x1, uv_y2], [uv_x2, uv_y2], [uv_x2, uv_y1], [uv_x1, uv_y1],
                // Other sides are uniform color of 0,0 pixel
                [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
                [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
                [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
                [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
                [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0],
            ];
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(mesh.clone()),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(image_handle.clone()),
                        reflectance: 0.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(x as f32 * 100.0, y as f32 * 100.0, 0.0)
                        .with_scale(Vec3::new(93.0, 93.0, 5.)),
                    ..default()
                })
                .insert(Id(x, y));
        }
    }
    commands.insert_resource(AmbientLight {
        brightness: 3.0,
        ..default()
    });
}

fn update(time: Res<Time>, mut query: Query<(&mut Transform, &Id)>) {
    for (mut tf, id) in query.iter_mut() {
        if id.0 == -2 && id.1.abs() == 2 {
            tf.rotate(Quat::from_axis_angle(Vec3::Y, time.delta_seconds()))
        }
        if id.0 == 2 && id.1.abs() == 2 {
            tf.rotate(Quat::from_axis_angle(Vec3::X, time.delta_seconds()))
        }
    }
}
