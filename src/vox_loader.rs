//! VOX file loader and hot reload system
//!
//! Loads MagicaVoxel .vox files directly and converts them to Bevy meshes.
//! Supports hot reloading: when a .vox file is modified, the model is automatically updated.
//! Also handles texture atlas hot reloading.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use dot_vox::DotVoxData;
use std::collections::HashMap;
use std::path::Path;

use crossbeam_channel::{unbounded, Receiver};
use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Resource to store loaded VOX meshes
#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct VoxMeshes {
    pub meshes: HashMap<String, Handle<Mesh>>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
}

/// Resource to store the current block texture atlas
#[derive(Resource, Default)]
pub struct BlockTextureAtlas {
    /// Handle to the current atlas texture
    pub texture: Handle<Image>,
    /// Path of the currently loaded atlas (for reload detection)
    pub current_path: String,
    /// Generation counter for hot reload
    pub generation: u32,
}

/// Resource to store the voxel array texture for block rendering
/// Uses 2D array texture for proper tiling with greedy meshing
#[derive(Resource, Default)]
pub struct VoxelArrayTexture {
    /// Handle to the array texture image
    pub texture: Handle<Image>,
    /// Number of layers in the array texture
    pub layer_count: u32,
    /// Whether the texture has been loaded
    pub is_loaded: bool,
}

/// Resource to track file changes
#[derive(Resource)]
pub struct VoxFileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

/// Event sent when a VOX file is modified
#[derive(Event)]
pub struct VoxFileChanged {
    pub path: String,
}

/// Event sent when the texture atlas is modified
#[derive(Event)]
pub struct TextureAtlasChanged {
    pub path: String,
}

/// Plugin for VOX file loading and hot reload
pub struct VoxLoaderPlugin;

impl Plugin for VoxLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxMeshes>()
            .init_resource::<BlockTextureAtlas>()
            .init_resource::<VoxelArrayTexture>()
            .add_event::<VoxFileChanged>()
            .add_event::<TextureAtlasChanged>()
            .add_systems(
                Startup,
                (
                    setup_file_watcher,
                    load_initial_vox_models,
                    load_initial_texture_atlas,
                    load_voxel_array_texture,
                ),
            )
            .add_systems(
                Update,
                (
                    check_file_changes,
                    handle_vox_reload,
                    handle_texture_atlas_reload,
                    configure_array_texture,
                ),
            );
    }
}

/// Helper to load a VOX file and add to meshes
fn try_load_vox(path: &str, meshes: &mut Assets<Mesh>) -> Option<Handle<Mesh>> {
    let full_path = Path::new(path);
    if full_path.exists() {
        if let Some((mesh, _)) = load_vox_mesh(full_path) {
            let handle = meshes.add(mesh);
            tracing::info!("Loaded VOX: {}", path);
            return Some(handle);
        }
    }
    None
}

/// Load initial VOX models at startup
fn load_initial_vox_models(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut models: ResMut<crate::components::MachineModels>,
) {
    // Machine VOX models
    models.vox_miner = try_load_vox("assets/models/machines/miner.vox", &mut meshes);
    models.vox_furnace = try_load_vox("assets/models/machines/furnace.vox", &mut meshes);
    models.vox_crusher = try_load_vox("assets/models/machines/crusher.vox", &mut meshes);
    models.vox_assembler = try_load_vox("assets/models/machines/assembler.vox", &mut meshes);
    models.vox_generator = try_load_vox("assets/models/machines/generator.vox", &mut meshes);
    models.vox_inserter = try_load_vox("assets/models/machines/inserter.vox", &mut meshes);
    models.vox_storage = try_load_vox("assets/models/machines/storage.vox", &mut meshes);
    models.vox_splitter_machine =
        try_load_vox("assets/models/machines/splitter_machine.vox", &mut meshes);

    // Conveyor VOX models
    models.vox_conveyor_straight =
        try_load_vox("assets/models/machines/conveyor/straight.vox", &mut meshes);
    models.vox_conveyor_corner_left = try_load_vox(
        "assets/models/machines/conveyor/corner_left.vox",
        &mut meshes,
    );
    models.vox_conveyor_corner_right = try_load_vox(
        "assets/models/machines/conveyor/corner_right.vox",
        &mut meshes,
    );
    models.vox_conveyor_t_junction = try_load_vox(
        "assets/models/machines/conveyor/t_junction.vox",
        &mut meshes,
    );
    models.vox_conveyor_splitter =
        try_load_vox("assets/models/machines/conveyor/splitter.vox", &mut meshes);

    // Create a shared material for VOX models (uses vertex colors)
    models.vox_material = Some(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    }));

    let vox_count = [
        models.vox_miner.is_some(),
        models.vox_furnace.is_some(),
        models.vox_crusher.is_some(),
        models.vox_assembler.is_some(),
        models.vox_generator.is_some(),
        models.vox_inserter.is_some(),
        models.vox_storage.is_some(),
        models.vox_splitter_machine.is_some(),
        models.vox_conveyor_straight.is_some(),
        models.vox_conveyor_corner_left.is_some(),
        models.vox_conveyor_corner_right.is_some(),
        models.vox_conveyor_t_junction.is_some(),
        models.vox_conveyor_splitter.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    tracing::info!("VOX models loaded: {}/13", vox_count);
}

/// Handle VOX file reload when files change
fn handle_vox_reload(
    mut events: EventReader<VoxFileChanged>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut models: ResMut<crate::components::MachineModels>,
) {
    for event in events.read() {
        let path = Path::new(&event.path);

        // Reload the changed VOX file
        if let Some((mesh, _)) = load_vox_mesh(path) {
            let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let handle = meshes.add(mesh);

            let reloaded = match filename {
                // Machine models
                "miner" => {
                    models.vox_miner = Some(handle);
                    true
                }
                "furnace" => {
                    models.vox_furnace = Some(handle);
                    true
                }
                "crusher" => {
                    models.vox_crusher = Some(handle);
                    true
                }
                "assembler" => {
                    models.vox_assembler = Some(handle);
                    true
                }
                "generator" => {
                    models.vox_generator = Some(handle);
                    true
                }
                "inserter" => {
                    models.vox_inserter = Some(handle);
                    true
                }
                "storage" => {
                    models.vox_storage = Some(handle);
                    true
                }
                "splitter_machine" => {
                    models.vox_splitter_machine = Some(handle);
                    true
                }
                // Conveyor models
                "straight" => {
                    models.vox_conveyor_straight = Some(handle);
                    true
                }
                "corner_left" => {
                    models.vox_conveyor_corner_left = Some(handle);
                    true
                }
                "corner_right" => {
                    models.vox_conveyor_corner_right = Some(handle);
                    true
                }
                "t_junction" => {
                    models.vox_conveyor_t_junction = Some(handle);
                    true
                }
                "splitter" => {
                    models.vox_conveyor_splitter = Some(handle);
                    true
                }
                _ => {
                    tracing::debug!("VOX file changed but not tracked: {}", filename);
                    false
                }
            };

            if reloaded {
                models.vox_generation += 1;
                tracing::info!(
                    "Hot reloaded: {}.vox (gen {})",
                    filename,
                    models.vox_generation
                );
            }
        }
    }
}

/// Find the best texture atlas file (alphabetically first block_atlas*.png)
fn find_best_texture_atlas() -> Option<std::path::PathBuf> {
    let textures_path = Path::new("assets/textures");
    if !textures_path.exists() {
        return None;
    }

    let mut candidates: Vec<_> = std::fs::read_dir(textures_path)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("block_atlas") && name_str.ends_with(".png")
        })
        .map(|e| e.path())
        .collect();

    candidates.sort();
    candidates.into_iter().next()
}

/// Load initial texture atlas at startup
fn load_initial_texture_atlas(
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<BlockTextureAtlas>,
) {
    if let Some(path) = find_best_texture_atlas() {
        let relative_path = path.strip_prefix("assets/").unwrap_or(&path);
        let path_str = relative_path.to_string_lossy().to_string();
        atlas.texture = asset_server.load(&path_str);
        atlas.current_path = path.to_string_lossy().to_string();
        tracing::info!("Loaded texture atlas: {}", atlas.current_path);
    } else {
        tracing::warn!("No block_atlas*.png found in assets/textures/");
    }
}

/// Handle texture atlas reload when files change
fn handle_texture_atlas_reload(
    mut events: EventReader<TextureAtlasChanged>,
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<BlockTextureAtlas>,
) {
    for event in events.read() {
        // Find the best atlas (in case a new one was added or the best one changed)
        if let Some(path) = find_best_texture_atlas() {
            let relative_path = path.strip_prefix("assets/").unwrap_or(&path);
            let path_str = relative_path.to_string_lossy().to_string();

            // Force reload by creating a new handle
            atlas.texture = asset_server.load(&path_str);
            atlas.current_path = path.to_string_lossy().to_string();
            atlas.generation += 1;
            tracing::info!(
                "Hot reloaded texture atlas: {} (gen {})",
                event.path,
                atlas.generation
            );
        }
    }
}

/// Number of texture layers in the block texture array
const BLOCK_TEXTURE_LAYERS: u32 = 8;

/// Load the voxel array texture at startup
fn load_voxel_array_texture(
    asset_server: Res<AssetServer>,
    mut array_tex: ResMut<VoxelArrayTexture>,
) {
    let path = "textures/block_textures_array.png";

    // Load as a normal image first, then convert to array texture
    array_tex.texture = asset_server.load(path);
    array_tex.layer_count = BLOCK_TEXTURE_LAYERS;

    tracing::info!(
        "Loading voxel array texture: {} ({} layers)",
        path,
        BLOCK_TEXTURE_LAYERS
    );
}

/// Configure array texture after it's loaded
/// Converts the stacked 2D image to a 2D array texture
fn configure_array_texture(
    mut array_tex: ResMut<VoxelArrayTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if array_tex.is_loaded {
        return;
    }

    // Check if texture is loaded
    if let Some(image) = images.get_mut(&array_tex.texture) {
        // Convert stacked 2D image to 2D array texture
        // The image is 16x128 (8 layers of 16x16 stacked vertically)
        image.reinterpret_stacked_2d_as_array(BLOCK_TEXTURE_LAYERS);

        // Set sampler to repeat for tiling
        image.sampler =
            bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                address_mode_u: bevy::image::ImageAddressMode::Repeat,
                address_mode_v: bevy::image::ImageAddressMode::Repeat,
                address_mode_w: bevy::image::ImageAddressMode::ClampToEdge,
                mag_filter: bevy::image::ImageFilterMode::Nearest,
                min_filter: bevy::image::ImageFilterMode::Nearest,
                ..default()
            });

        array_tex.is_loaded = true;
        tracing::info!(
            "Voxel array texture configured: {:?}, size: {:?}",
            image.texture_descriptor.dimension,
            image.size()
        );
    }
}

/// Set up file watcher for hot reload
fn setup_file_watcher(mut commands: Commands) {
    let (tx, rx) = unbounded();

    let watcher = recommended_watcher(move |res| {
        if let Err(e) = tx.send(res) {
            tracing::error!("Failed to send file event: {}", e);
        }
    });

    match watcher {
        Ok(mut w) => {
            // Watch the assets/models directory
            let models_path = Path::new("assets/models");
            if models_path.exists() {
                if let Err(e) = w.watch(models_path, RecursiveMode::Recursive) {
                    tracing::error!("Failed to watch models directory: {}", e);
                } else {
                    tracing::info!("VOX hot reload enabled: watching assets/models/");
                }
            }

            // Watch the assets/textures directory for texture atlas changes
            let textures_path = Path::new("assets/textures");
            if textures_path.exists() {
                if let Err(e) = w.watch(textures_path, RecursiveMode::NonRecursive) {
                    tracing::error!("Failed to watch textures directory: {}", e);
                } else {
                    tracing::info!("Texture hot reload enabled: watching assets/textures/");
                }
            }

            commands.insert_resource(VoxFileWatcher {
                _watcher: w,
                receiver: rx,
            });
        }
        Err(e) => {
            tracing::error!("Failed to create file watcher: {}", e);
        }
    }
}

/// Check for file changes and send events
fn check_file_changes(
    watcher: Option<Res<VoxFileWatcher>>,
    mut vox_events: EventWriter<VoxFileChanged>,
    mut texture_events: EventWriter<TextureAtlasChanged>,
) {
    let Some(watcher) = watcher else { return };

    while let Ok(Ok(event)) = watcher.receiver.try_recv() {
        use notify::EventKind;
        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
            for path in event.paths {
                let path_str = path.to_string_lossy().to_string();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if ext == "vox" {
                    tracing::info!("VOX file changed: {}", path_str);
                    vox_events.send(VoxFileChanged { path: path_str });
                } else if ext == "png" && filename.starts_with("block_atlas") {
                    tracing::info!("Texture atlas changed: {}", path_str);
                    texture_events.send(TextureAtlasChanged { path: path_str });
                }
            }
        }
    }
}

/// Load a VOX file and convert it to a Bevy Mesh
pub fn load_vox_mesh(path: &Path) -> Option<(Mesh, Vec<Color>)> {
    let data = std::fs::read(path).ok()?;
    let vox = dot_vox::load_bytes(&data).ok()?;
    vox_to_mesh(&vox)
}

/// Convert VOX data to Bevy Mesh with greedy meshing
#[allow(clippy::type_complexity)]
fn vox_to_mesh(vox: &DotVoxData) -> Option<(Mesh, Vec<Color>)> {
    let Some(model) = vox.models.first() else {
        tracing::warn!("VOXファイルにモデルが含まれていません");
        return None;
    };
    let palette = &vox.palette;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Create a 3D grid of voxels
    let mut grid: HashMap<(i32, i32, i32), u8> = HashMap::new();
    for v in &model.voxels {
        grid.insert((v.x as i32, v.y as i32, v.z as i32), v.i);
    }

    // Face definitions: (normal, vertices offsets)
    let faces: [([f32; 3], [[f32; 3]; 4], [i32; 3]); 6] = [
        // +X face
        (
            [1.0, 0.0, 0.0],
            [
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            [1, 0, 0],
        ),
        // -X face
        (
            [-1.0, 0.0, 0.0],
            [
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            [-1, 0, 0],
        ),
        // +Y face
        (
            [0.0, 1.0, 0.0],
            [
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            [0, 1, 0],
        ),
        // -Y face
        (
            [0.0, -1.0, 0.0],
            [
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
            ],
            [0, -1, 0],
        ),
        // +Z face
        (
            [0.0, 0.0, 1.0],
            [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            [0, 0, 1],
        ),
        // -Z face
        (
            [0.0, 0.0, -1.0],
            [
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0, 0, -1],
        ),
    ];

    // Calculate center offset for centering the model
    let (min_x, max_x, min_y, max_y, min_z, max_z) = model.voxels.iter().fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_x, max_x, min_y, max_y, min_z, max_z), v| {
            (
                min_x.min(v.x as i32),
                max_x.max(v.x as i32),
                min_y.min(v.y as i32),
                max_y.max(v.y as i32),
                min_z.min(v.z as i32),
                max_z.max(v.z as i32),
            )
        },
    );

    let center_x = (min_x + max_x) as f32 / 2.0;
    let center_y = (min_y + max_y) as f32 / 2.0;
    let center_z = (min_z + max_z) as f32 / 2.0;

    // Scale factor (16 voxels = 1 game unit)
    let scale = 1.0 / 16.0;

    // Generate faces for each voxel
    for v in &model.voxels {
        let (x, y, z) = (v.x as i32, v.y as i32, v.z as i32);
        let color_idx = v.i as usize;
        let color = &palette[color_idx];
        let rgba = [
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ];

        for (normal, verts, neighbor_offset) in &faces {
            let neighbor = (
                x + neighbor_offset[0],
                y + neighbor_offset[1],
                z + neighbor_offset[2],
            );

            // Only add face if neighbor is empty
            if !grid.contains_key(&neighbor) {
                let base_idx = positions.len() as u32;

                for vert in verts {
                    positions.push([
                        (x as f32 + vert[0] - center_x) * scale,
                        (z as f32 + vert[2] - center_z) * scale, // Swap Y and Z for Bevy's coordinate system
                        (y as f32 + vert[1] - center_y) * scale,
                    ]);
                    normals.push([normal[0], normal[2], normal[1]]);
                    colors.push(rgba);
                }

                // Two triangles per face
                indices.extend_from_slice(&[
                    base_idx,
                    base_idx + 1,
                    base_idx + 2,
                    base_idx,
                    base_idx + 2,
                    base_idx + 3,
                ]);
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    // Extract unique colors for reference
    let unique_colors: Vec<Color> = palette
        .iter()
        .map(|c| {
            Color::srgba(
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
                c.a as f32 / 255.0,
            )
        })
        .collect();

    Some((mesh, unique_colors))
}

/// Load all VOX files from assets/models and convert them
#[allow(dead_code)]
pub fn load_all_vox_models(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    vox_meshes: &mut VoxMeshes,
) {
    let models_path = Path::new("assets/models");
    if !models_path.exists() {
        tracing::warn!("assets/models directory not found");
        return;
    }

    load_vox_recursive(models_path, meshes, materials, vox_meshes);
}

#[allow(dead_code)]
fn load_vox_recursive(
    dir: &Path,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    vox_meshes: &mut VoxMeshes,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_vox_recursive(&path, meshes, materials, vox_meshes);
        } else if path.extension().is_some_and(|ext| ext == "vox") {
            if let Some((mesh, _colors)) = load_vox_mesh(&path) {
                let key = path.to_string_lossy().to_string();
                let mesh_handle = meshes.add(mesh);
                let material_handle = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    ..default()
                });

                tracing::info!("Loaded VOX: {}", key);
                vox_meshes.meshes.insert(key.clone(), mesh_handle);
                vox_meshes.materials.insert(key, material_handle);
            }
        }
    }
}
