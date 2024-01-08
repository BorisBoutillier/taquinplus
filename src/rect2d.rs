//! Create a custom material to draw basic lines in 3D

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
//fn setup(
//    mut commands: Commands,
//    mut meshes: ResMut<Assets<Mesh>>,
//    mut materials: ResMut<Assets<Rect2dMaterial>>,
//) {
//    // Spawn a line strip that goes from point to point
//    commands.spawn(MaterialMeshBundle {
//        mesh: meshes.add(Mesh::from(LineStrip {
//            points: vec![
//                Vec3::ZERO,
//                Vec3::new(1.0, 1.0, 0.0),
//                Vec3::new(1.0, 0.0, 0.0),
//            ],
//        })),
//        transform: Transform::from_xyz(0.5, 0.0, 0.0),
//        material: materials.add(Rect2dMaterial { color: Color::BLUE }),
//        ..default()
//    });
//
//    // camera
//    commands.spawn(Camera3dBundle {
//        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//        ..default()
//    });
//}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct Rect2dMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl Material for Rect2dMaterial {
    fn fragment_shader() -> ShaderRef {
        "rect2d_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of points that will have a line drawn between each consecutive points
#[derive(Debug, Clone)]
pub struct Rect2d {
    pub x_length: f32,
    pub y_length: f32,
}

impl From<Rect2d> for Mesh {
    fn from(rect2d: Rect2d) -> Self {
        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        let x_half = rect2d.x_length / 2.;
        let y_half = rect2d.y_length / 2.;
        let points = vec![
            Vec3::new(-x_half, -y_half, 0.),
            Vec3::new(-x_half, y_half, 0.),
            Vec3::new(x_half, y_half, 0.),
            Vec3::new(x_half, -y_half, 0.),
            Vec3::new(-x_half, -y_half, 0.),
        ];
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
        mesh
    }
}