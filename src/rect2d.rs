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
