// Voxel block shader using 2D array texture
// Enables proper texture tiling with greedy meshing

#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var block_textures: texture_2d_array<f32>;
@group(2) @binding(1) var block_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // UV.x and UV.y are texture coordinates (can be > 1 for tiling)
    // UV.z (stored in uv_b.x) is the texture layer index
    let layer = u32(mesh.uv_b.x);

    // Use fract() for seamless tiling across greedy-meshed quads
    let tiled_uv = fract(mesh.uv);

    // Sample from the appropriate texture layer
    let color = textureSample(block_textures, block_sampler, tiled_uv, layer);

    return color;
}
