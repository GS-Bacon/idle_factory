use bevy::prelude::*;
use std::collections::HashMap;
use crate::gameplay::grid::{Machine, SimulationGrid, ItemSlot};

#[derive(Component)]
pub struct VisualItem {
    pub unique_id: u64,
}

pub fn update_visual_items(
    mut commands: Commands,
    grid: Res<SimulationGrid>,
    visuals: Query<(Entity, &VisualItem)>,
    mut transforms: Query<&mut Transform>,
    asset_server: Res<AssetServer>,
) {
    let mut visual_map: HashMap<u64, Entity> = visuals.iter().map(|(e, v)| (v.unique_id, e)).collect();
    
    for (pos, machine) in &grid.machines {
        
        let mut inventory_to_render: Vec<&ItemSlot> = Vec::new();
        let is_conveyor = match &machine.machine_type {
            Machine::Conveyor(c) => {
                inventory_to_render.extend(c.inventory.iter());
                true
            }
            Machine::Assembler(a) => {
                inventory_to_render.extend(a.input_inventory.iter());
                inventory_to_render.extend(a.output_inventory.iter());
                false
            }
            Machine::Miner(_) => false,
        };
        
        for item in inventory_to_render {
            let final_pos = if is_conveyor {
                let center_pos = pos.as_vec3() + Vec3::new(0.5, 0.2, 0.5);
                let current_dir = machine.orientation.to_ivec3().as_vec3();
                let from_dir = item.from_direction.unwrap_or(machine.orientation).to_ivec3().as_vec3();

                if from_dir == current_dir {
                    let offset = current_dir * (item.progress - 0.5);
                    center_pos + offset
                } else {
                    if item.progress < 0.5 {
                        let start_point = center_pos - (from_dir * 0.5);
                        let t = item.progress * 2.0;
                        start_point.lerp(center_pos, t)
                    } else {
                        let mid_point = center_pos;
                        let end_point = center_pos + (current_dir * 0.5);
                        let t = (item.progress - 0.5) * 2.0;
                        mid_point.lerp(end_point, t)
                    }
                }
            } else {
                 pos.as_vec3() + Vec3::new(0.5, 1.1, 0.5)
            };

            if let Some(entity) = visual_map.remove(&item.unique_id) {
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    transform.translation = final_pos;
                }
            } else {
                let model_path = match item.item_id.as_str() {
                    "raw_ore" => "models/items/ore.glb#Scene0",
                    "ingot" => "models/items/cube.glb#Scene0",
                    _ => "models/items/cube.glb#Scene0",
                };
                
                let scale = Vec3::splat(0.75); 

                commands.spawn((
                    SceneRoot(asset_server.load(model_path)),
                    Transform::from_translation(final_pos).with_scale(scale),
                    GlobalTransform::default(), // Required for scenes
                    VisualItem { unique_id: item.unique_id },
                ));
            }
        }
    }

    for entity in visual_map.values() {
        commands.entity(*entity).despawn_recursive();
    }
}