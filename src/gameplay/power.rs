use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::gameplay::grid::{Machine, MachineInstance, SimulationGrid};

// --- Components ---

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PowerNode {
    pub id: u32, // Unique ID for this node
    pub group_id: Option<u32>, // Which network group it belongs to
}

#[derive(Component, Debug)]
pub struct PowerSource {
    pub capacity: f32, // Max stress this source can handle
    pub current_speed: f32, // Current rotation speed (e.g., RPM)
}

#[derive(Component, Debug)]
pub struct PowerConsumer {
    pub stress_impact: f32, // Stress demand when active
    pub is_active: bool, // Whether the consumer is currently trying to draw power
    pub current_speed_received: f32, // Actual speed received from the network
}

#[derive(Component, Debug)]
pub struct Shaft {
    pub stress_resistance: f32, // Resistance to breaking under stress
}

// --- Resources ---

/// Represents the overall power network as a graph
#[derive(Resource, Default)]
pub struct PowerNetworkGraph {
    // Adjacency list: NodeId -> Set of connected NodeIds
    pub adjacencies: HashMap<u32, HashSet<u32>>,
    pub node_entity_map: HashMap<u32, Entity>, // Map node ID to its Bevy Entity
    pub next_node_id: u32, // Simple counter for unique node IDs
}

/// Represents a connected group of power machines
#[derive(Resource, Default)]
pub struct PowerNetworkGroups {
    pub groups: HashMap<u32, NetworkGroup>, // Group ID -> NetworkGroup
    pub next_group_id: u32,
}

#[derive(Debug, Default)]
pub struct NetworkGroup {
    pub nodes: HashSet<u32>, // All node IDs in this group
    pub total_stress_demand: f32,
    pub total_source_capacity: f32,
    pub is_overstressed: bool,
    pub ideal_speed: f32, // Ideal operating speed for this group
}

// --- Systems ---

/// System to initialize PowerNodes when a power-related entity is spawned.
pub fn spawn_power_node_system(
    mut commands: Commands,
    mut graph: ResMut<PowerNetworkGraph>,
    query_new_sources: Query<Entity, (With<PowerSource>, Without<PowerNode>)>,
    query_new_consumers: Query<Entity, (With<PowerConsumer>, Without<PowerNode>)>,
    query_new_shafts: Query<Entity, (With<Shaft>, Without<PowerNode>)>,
) {
    for entity in query_new_sources.iter().chain(query_new_consumers.iter()).chain(query_new_shafts.iter()) {
        let node_id = graph.next_node_id;
        commands.entity(entity).insert(PowerNode { id: node_id, group_id: None });
        graph.node_entity_map.insert(node_id, entity);
        graph.next_node_id += 1;
        // Connections would be handled by another system based on spatial data
    }
}

pub fn spawn_power_entities_from_grid(
    mut commands: Commands,
    mut grid: ResMut<SimulationGrid>,
    mut power_graph: ResMut<PowerNetworkGraph>,
) {
    let mut to_spawn_power_entities = Vec::<(IVec3, MachineInstance)>::new();

    // Collect machines that need a power entity
    for (pos, instance) in grid.machines.iter() {
        if instance.power_node.is_none() {
            // Check if it's a power-related machine type
            match instance.machine_type {
                Machine::Miner(_) | Machine::Assembler(_) => { // These are consumers
                    to_spawn_power_entities.push((*pos, instance.clone()));
                },
                // Add Shaft and PowerSource types later
                _ => {} // Not power related or already handled
            }
        }
    }

    for (pos, mut instance) in to_spawn_power_entities {
        let machine_entity = commands.spawn(PbrBundle::default()).id(); // Consider using a better bundle/spawn for future rendering

        // Add PowerNode component
        let node_id = power_graph.next_node_id;
        commands.entity(machine_entity).insert(PowerNode { id: node_id, group_id: None });
        power_graph.node_entity_map.insert(node_id, machine_entity);
        power_graph.next_node_id += 1;

        // Add specific power component based on machine_type
        match instance.machine_type {
            Machine::Miner(_) => {
                commands.entity(machine_entity).insert(PowerConsumer {
                    stress_impact: 10.0, // Example value
                    is_active: true,
                    current_speed_received: 0.0,
                });
            },
            Machine::Assembler(_) => {
                commands.entity(machine_entity).insert(PowerConsumer {
                    stress_impact: 20.0, // Example value
                    is_active: true,
                    current_speed_received: 0.0,
                });
            },
            _ => {} // Should not happen if filtered correctly above
        }

        // Update the MachineInstance in the grid with the new entity
        instance.power_node = Some(machine_entity);
        // Need to update the grid with the modified instance
        grid.machines.insert(pos, instance);
    }
}

/// System to update the PowerNetworkGraph based on entity positions and connections.
pub fn update_power_graph_system(
    mut power_graph: ResMut<PowerNetworkGraph>,
    grid: Res<SimulationGrid>,
    query_power_nodes: Query<&PowerNode>,
) {
    // Clear existing adjacencies to rebuild the graph
    power_graph.adjacencies.clear();

    for (pos, machine_instance) in grid.machines.iter() {
        if let Some(machine_power_entity) = machine_instance.power_node {
            // Ensure the entity still exists and has a PowerNode
            if let Ok(current_power_node) = query_power_nodes.get(machine_power_entity) {
                // Determine potential connection points based on machine type and orientation
                let connection_directions = match &machine_instance.machine_type {
                    // For now, assume all power-consuming machines connect from their "back"
                    // This means the side opposite to their orientation
                    Machine::Miner(_) | Machine::Assembler(_) => {
                        vec![machine_instance.orientation.opposite()]
                    },
                    // Add Shaft connections later (e.g., all 4 cardinal directions)
                    _ => vec![],
                };

                for dir in connection_directions {
                    let neighbor_pos = *pos + dir.to_ivec3();
                    if let Some(neighbor_instance) = grid.machines.get(&neighbor_pos) {
                        if let Some(neighbor_power_entity) = neighbor_instance.power_node {
                            // Ensure the neighbor entity also exists and has a PowerNode
                            if let Ok(neighbor_power_node) = query_power_nodes.get(neighbor_power_entity) {
                                // Add edge in both directions
                                power_graph.adjacencies
                                    .entry(current_power_node.id)
                                    .or_default()
                                    .insert(neighbor_power_node.id);
                                power_graph.adjacencies
                                    .entry(neighbor_power_node.id)
                                    .or_default()
                                    .insert(current_power_node.id);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn detect_network_groups_system(
    power_network: Res<PowerNetworkGraph>,
    mut power_groups: ResMut<PowerNetworkGroups>,
    mut query_nodes: Query<&mut PowerNode>,
) {
    power_groups.groups.clear();
    let mut visited: HashSet<u32> = HashSet::new();
    let mut current_group_id = power_groups.next_group_id;

    let all_node_ids: Vec<u32> = power_network.adjacencies.keys().cloned().collect();

    for &start_node_id in all_node_ids.iter() {
        if !visited.contains(&start_node_id) {
            let mut q: Vec<u32> = Vec::new();
            q.push(start_node_id);
            visited.insert(start_node_id);

            let mut new_group = NetworkGroup::default();
            new_group.nodes.insert(start_node_id);

            let mut head = 0;
            while head < q.len() {
                let node_id = q[head];
                head += 1;

                if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                    if let Ok(mut power_node) = query_nodes.get_mut(*entity) {
                        power_node.group_id = Some(current_group_id);
                    }
                }

                if let Some(neighbors) = power_network.adjacencies.get(&node_id) {
                    for &neighbor_id in neighbors.iter() {
                        if !visited.contains(&neighbor_id) {
                            visited.insert(neighbor_id);
                            q.push(neighbor_id);
                            new_group.nodes.insert(neighbor_id);
                        }
                    }
                }
            }
            power_groups.groups.insert(current_group_id, new_group);
            current_group_id += 1;
        }
    }
    power_groups.next_group_id = current_group_id;
}


pub fn calculate_power_states_system(
    power_network: Res<PowerNetworkGraph>,
    mut power_groups: ResMut<PowerNetworkGroups>,
    query_sources: Query<(&PowerNode, &PowerSource)>,
    query_consumers: Query<(&PowerNode, &PowerConsumer)>,
    mut query_sources_mut: Query<&mut PowerSource>,
    mut query_consumers_mut: Query<&mut PowerConsumer>,
) {
    for (_group_id, group) in power_groups.groups.iter_mut() {
        let mut current_total_stress = 0.0;
        let mut current_total_capacity = 0.0;

        for &node_id in group.nodes.iter() {
            if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                if let Ok((_node, consumer)) = query_consumers.get(*entity) {
                    if consumer.is_active {
                        current_total_stress += consumer.stress_impact;
                    }
                }
                if let Ok((_node, source)) = query_sources.get(*entity) {
                    current_total_capacity += source.capacity;
                }
            }
        }

        group.total_stress_demand = current_total_stress;
        group.total_source_capacity = current_total_capacity;
        group.is_overstressed = current_total_stress > current_total_capacity;
        group.ideal_speed = if group.is_overstressed { 0.0 } else { 1.0 };

        for &node_id in group.nodes.iter() {
            if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                if let Ok(mut consumer) = query_consumers_mut.get_mut(*entity) {
                    consumer.current_speed_received = group.ideal_speed;
                    if group.is_overstressed {
                        consumer.is_active = false;
                    }
                }
                if let Ok(mut source) = query_sources_mut.get_mut(*entity) {
                    source.current_speed = group.ideal_speed;
                }
            }
        }
    }
}

// Add a Bevy plugin for the power system
pub struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerNetworkGraph>()
           .init_resource::<PowerNetworkGroups>()
           .add_systems(Update, spawn_power_node_system)
           .add_systems(Update, spawn_power_entities_from_grid.run_if(|mut reader: EventReader<crate::gameplay::building::MachinePlacedEvent>| reader.read().next().is_some()))
           .add_systems(Update, update_power_graph_system) // Add the system to update the power graph
           .add_systems(Update, detect_network_groups_system) // Add the system to detect network groups
           .add_systems(Update, calculate_power_states_system); // Add the system to calculate power states
    }
}