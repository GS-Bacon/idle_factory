//! VoxelMaterial - Custom material using 2D array texture for block rendering
//!
//! This material enables proper texture tiling with greedy meshing by using
//! a texture array (one layer per block type) instead of an atlas.

use bevy::{
    asset::Asset,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

/// Custom material for voxel/block rendering using 2D array textures.
///
/// Uses UV.z as the texture layer index for seamless tiling with greedy meshing.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    /// The texture array containing all block textures (one layer per block type)
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Enable backface culling (standard for solid blocks)
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(Some(ref mut state)) = fragment.targets.first_mut() {
                // Keep default blend state (opaque)
                state.blend = None;
            }
        }
        Ok(())
    }
}
