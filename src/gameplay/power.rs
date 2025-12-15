use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

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

/// System to update the PowerNetworkGraph based on entity positions and connections.
/// This would be triggered by building/destroying power-related structures.
/// This is a placeholder for actual spatial connection logic.
pub fn update_power_graph_system(
    mut graph: ResMut<PowerNetworkGraph>,
    // This query would need to check entity positions and link them
    // For a voxel game, this usually involves checking adjacent grid cells
    // This example is simplified and doesn't contain actual spatial logic.
    mut has_changed: Local<bool>, // A flag to indicate if connections changed
) {
    // In a real game, this would re-evaluate connections based on grid positions.
    // For now, let's just make a dummy connection for demonstration.
    if !*has_changed && graph.node_entity_map.len() >= 2 {
        let mut node_ids: Vec<u32> = graph.node_entity_map.keys().cloned().collect();
        node_ids.sort(); // Ensure consistent ordering
        if let (Some(id1), Some(id2)) = (node_ids.get(0), node_ids.get(1)) {
            graph.adjacencies.entry(*id1).or_default().insert(*id2);
            graph.adjacencies.entry(*id2).or_default().insert(*id1);
            info!("Dummy: Connected node {} and {}", id1, id2);
            *has_changed = true;
        }
    }
}


/// System to detect connected components and create/update NetworkGroups.
pub fn detect_network_groups_system(
    power_network: Res<PowerNetworkGraph>,
    mut power_groups: ResMut<PowerNetworkGroups>,
    mut query_nodes: Query<&mut PowerNode>,
) {
    power_groups.groups.clear(); // Clear existing groups
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

                // Update PowerNode component with new group_id
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


/// Fixed timestep system to calculate stress and update power states.
pub fn calculate_power_states_system(
    power_network: Res<PowerNetworkGraph>, // Read-only access to graph
    mut power_groups: ResMut<PowerNetworkGroups>,
    query_sources: Query<(&PowerNode, &PowerSource)>,
    query_consumers: Query<(&PowerNode, &PowerConsumer)>,
    mut query_sources_mut: Query<&mut PowerSource>,
    mut query_consumers_mut: Query<&mut PowerConsumer>,
) {
    for (_group_id, group) in power_groups.groups.iter_mut() {
        let mut current_total_stress = 0.0;
        let mut current_total_capacity = 0.0;

        // Sum stress and capacity for this group
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
        group.ideal_speed = if group.is_overstressed { 0.0 } else { 1.0 }; // Example: 1.0 for full speed, 0.0 for stopped

        // Propagate state to individual machines
        for &node_id in group.nodes.iter() {
            if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                // Update consumers
                if let Ok(mut consumer) = query_consumers_mut.get_mut(*entity) {
                    consumer.current_speed_received = group.ideal_speed;
                    // If overstressed, consumers might automatically deactivate or slow down
                    if group.is_overstressed {
                        consumer.is_active = false; // Example: Force deactivation
                    }
                }
                // Update sources
                if let Ok(mut source) = query_sources_mut.get_mut(*entity) {
                    source.current_speed = group.ideal_speed;
                }
            }
        }
    }
}

pub struct PowerPlugin;

impl Plugin for PowerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PowerNetworkGraph>()
            .init_resource::<PowerNetworkGroups>()
            .add_systems(FixedUpdate, (
                spawn_power_node_system,
                update_power_graph_system,
                detect_network_groups_system,
                calculate_power_states_system,
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::core::TaskPoolPlugin;
    use bevy::ecs::schedule::Schedule; // For app.world_mut().run_schedule
    use bevy::time::{Fixed, Time, TimePlugin};
    // Removed use std::time::Duration;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(TaskPoolPlugin::default()); // Needed for some Bevy internals
        app.add_plugins(TimePlugin); // Provides Time and Fixed resources
        app.add_plugins(PowerPlugin); // Our plugin

        // Initialize Time<Fixed> period
        app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
        
        app
    }

    #[test]
    fn test_power_node_spawn_and_grouping() {
        let mut app = setup_app();

        let source_entity = app.world_mut().spawn(PowerSource { capacity: 100.0, current_speed: 0.0 }).id();
        let consumer_entity = app.world_mut().spawn(PowerConsumer { stress_impact: 50.0, is_active: true, current_speed_received: 0.0 }).id();
        
        // Ensure commands are applied
        app.world_mut().flush();

        // Create a temporary schedule to run the systems.
        // This is more robust for unit testing specific system groups.
        let mut test_schedule = Schedule::new(FixedUpdate);
        test_schedule.add_systems((
            spawn_power_node_system,
            update_power_graph_system,
            detect_network_groups_system,
            calculate_power_states_system,
        ));
        
        // Run the schedule. This will execute the systems and flush commands.
        test_schedule.run(&mut app.world_mut());

        // Assertions
        let power_node_source = app.world().get::<PowerNode>(source_entity).expect("PowerNode should be added to source");
        let power_node_consumer = app.world().get::<PowerNode>(consumer_entity).expect("PowerNode should be added to consumer");

        assert_eq!(power_node_source.id, 0);
        assert_eq!(power_node_consumer.id, 1);

        let power_groups = app.world().resource::<PowerNetworkGroups>();
        assert_eq!(power_groups.groups.len(), 1);

        let group = power_groups.groups.get(&power_node_source.group_id.unwrap()).expect("Group should exist");
        assert!(group.nodes.contains(&power_node_source.id));
        assert!(group.nodes.contains(&power_node_consumer.id));

        let power_graph = app.world().resource::<PowerNetworkGraph>();
        assert!(power_graph.adjacencies.get(&0).unwrap().contains(&1));
        assert!(power_graph.adjacencies.get(&1).unwrap().contains(&0));

        let source = app.world().get::<PowerSource>(source_entity).unwrap();
        let consumer = app.world().get::<PowerConsumer>(consumer_entity).unwrap();

        assert!(!group.is_overstressed);
        assert_eq!(group.ideal_speed, 1.0);
        assert_eq!(source.current_speed, 1.0);
        assert_eq!(consumer.current_speed_received, 1.0);
        assert_eq!(group.total_stress_demand, 50.0);
        assert_eq!(group.total_source_capacity, 100.0);
    }

    #[test]
    fn test_power_node_overstressed_condition() {
        let mut app = setup_app();

        // Spawn entities
        let source_entity = app.world_mut().spawn(PowerSource { capacity: 10.0, current_speed: 0.0 }).id();
        let consumer_entity = app.world_mut().spawn(PowerConsumer { stress_impact: 50.0, is_active: true, current_speed_received: 0.0 }).id();

        // Ensure commands are applied
        app.world_mut().flush();

        let mut test_schedule = Schedule::new(FixedUpdate);
        test_schedule.add_systems((
            spawn_power_node_system,
            update_power_graph_system,
            detect_network_groups_system,
            calculate_power_states_system,
        ));
        test_schedule.run(&mut app.world_mut());

        // Assertions
        let power_node_source = app.world().get::<PowerNode>(source_entity).expect("PowerNode should be added to source");
        
        let power_groups = app.world().resource::<PowerNetworkGroups>();
        let group = power_groups.groups.get(&power_node_source.group_id.unwrap()).expect("Group should exist");
        
        assert!(group.is_overstressed);
        assert_eq!(group.ideal_speed, 0.0);

        let source = app.world().get::<PowerSource>(source_entity).unwrap();
        let consumer = app.world().get::<PowerConsumer>(consumer_entity).unwrap();

        assert_eq!(source.current_speed, 0.0);
        assert_eq!(consumer.current_speed_received, 0.0);
        assert!(!consumer.is_active);
    }
}
