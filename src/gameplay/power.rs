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

/// 回路遮断機コンポーネント（Satisfactory風）
/// 電力過負荷時に自動でトリップし、ネットワークを保護する
#[derive(Component, Debug, Clone)]
pub struct CircuitBreaker {
    /// 最大負荷（これを超えるとトリップ）
    pub max_load: f32,
    /// 現在トリップしているかどうか
    pub is_tripped: bool,
    /// 自動リセットを有効にするか
    pub auto_reset: bool,
    /// 自動リセットまでの待機時間（秒）
    pub reset_delay: f32,
    /// トリップしてからの経過時間
    pub time_since_trip: f32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            max_load: 500.0,
            is_tripped: false,
            auto_reset: false,
            reset_delay: 5.0,
            time_since_trip: 0.0,
        }
    }
}

impl CircuitBreaker {
    pub fn new(max_load: f32) -> Self {
        Self {
            max_load,
            ..Default::default()
        }
    }

    /// 手動でリセット
    pub fn reset(&mut self) {
        self.is_tripped = false;
        self.time_since_trip = 0.0;
    }

    /// トリップさせる
    pub fn trip(&mut self) {
        if !self.is_tripped {
            self.is_tripped = true;
            self.time_since_trip = 0.0;
        }
    }
}

/// 電力スイッチコンポーネント
/// 手動で電力の流れを制御
#[derive(Component, Debug, Clone, Default)]
pub struct PowerSwitch {
    /// スイッチがONかどうか
    pub is_on: bool,
}

impl PowerSwitch {
    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }
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

/// 電力ネットワークグループの状態
#[derive(Debug, Default)]
pub struct NetworkGroup {
    pub nodes: HashSet<u32>, // All node IDs in this group
    pub total_stress_demand: f32,
    pub total_source_capacity: f32,
    pub is_overstressed: bool,
    pub ideal_speed: f32, // Ideal operating speed for this group
    /// ヒューズ/回路遮断機によるトリップ状態
    pub is_tripped: bool,
    /// トリップした時刻（自動復帰用）
    pub tripped_at: Option<f64>,
    /// 過負荷履歴（直近5回）
    pub overload_history: Vec<f64>,
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
        if let (Some(id1), Some(id2)) = (node_ids.first(), node_ids.get(1)) {
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
    mut query_sources: Query<(&PowerNode, &mut PowerSource)>,
    mut query_consumers: Query<(&PowerNode, &mut PowerConsumer)>,
) {
    // First pass: collect stress and capacity data
    let mut group_data: Vec<(u32, f32, f32)> = Vec::new();

    for (group_id, group) in power_groups.groups.iter() {
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
        group_data.push((*group_id, current_total_stress, current_total_capacity));
    }

    // Update group stats and propagate to machines
    for (group_id, total_stress, total_capacity) in group_data {
        let is_overstressed = total_stress > total_capacity;
        let ideal_speed = if is_overstressed { 0.0 } else { 1.0 };

        if let Some(group) = power_groups.groups.get_mut(&group_id) {
            group.total_stress_demand = total_stress;
            group.total_source_capacity = total_capacity;
            group.is_overstressed = is_overstressed;
            group.ideal_speed = ideal_speed;

            // Propagate state to individual machines
            for &node_id in group.nodes.iter() {
                if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                    if let Ok((_node, mut consumer)) = query_consumers.get_mut(*entity) {
                        consumer.current_speed_received = ideal_speed;
                        if is_overstressed {
                            consumer.is_active = false;
                        }
                    }
                    if let Ok((_node, mut source)) = query_sources.get_mut(*entity) {
                        source.current_speed = ideal_speed;
                    }
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
    use bevy::MinimalPlugins;
    use bevy::app::App;

    // Test plugin that runs power systems in Update instead of FixedUpdate
    struct TestPowerPlugin;

    impl Plugin for TestPowerPlugin {
        fn build(&self, app: &mut App) {
            app
                .init_resource::<PowerNetworkGraph>()
                .init_resource::<PowerNetworkGroups>()
                .add_systems(Update, (
                    spawn_power_node_system,
                    update_power_graph_system,
                    detect_network_groups_system,
                    calculate_power_states_system,
                ).chain());
        }
    }

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(TestPowerPlugin);
        app
    }

    #[test]
    fn test_power_node_spawn_and_grouping() {
        let mut app = setup_app();

        let source_entity = app.world_mut().spawn(PowerSource { capacity: 100.0, current_speed: 0.0 }).id();
        let consumer_entity = app.world_mut().spawn(PowerConsumer { stress_impact: 50.0, is_active: true, current_speed_received: 0.0 }).id();

        // Run multiple updates to ensure all systems have executed and commands are applied
        for _ in 0..5 {
            app.update();
        }

        // Assertions - PowerNode should be added
        let power_node_source = app.world().get::<PowerNode>(source_entity).expect("PowerNode should be added to source");
        let power_node_consumer = app.world().get::<PowerNode>(consumer_entity).expect("PowerNode should be added to consumer");

        assert_eq!(power_node_source.id, 0);
        assert_eq!(power_node_consumer.id, 1);

        // Check that groups were formed
        let power_groups = app.world().resource::<PowerNetworkGroups>();
        assert!(power_groups.groups.len() >= 1, "At least one group should exist");

        // Group ID should be set after systems run
        if let Some(group_id) = power_node_source.group_id {
            let group = power_groups.groups.get(&group_id).expect("Group should exist");
            assert!(group.nodes.contains(&power_node_source.id));
            assert!(group.nodes.contains(&power_node_consumer.id));

            assert!(!group.is_overstressed);
            assert_eq!(group.ideal_speed, 1.0);
            assert_eq!(group.total_stress_demand, 50.0);
            assert_eq!(group.total_source_capacity, 100.0);
        }

        let source = app.world().get::<PowerSource>(source_entity).unwrap();
        let consumer = app.world().get::<PowerConsumer>(consumer_entity).unwrap();

        assert_eq!(source.current_speed, 1.0);
        assert_eq!(consumer.current_speed_received, 1.0);
    }

    #[test]
    fn test_power_node_overstressed_condition() {
        let mut app = setup_app();

        let source_entity = app.world_mut().spawn(PowerSource { capacity: 10.0, current_speed: 0.0 }).id();
        let consumer_entity = app.world_mut().spawn(PowerConsumer { stress_impact: 50.0, is_active: true, current_speed_received: 0.0 }).id();

        // Run updates to process systems
        // Behavior:
        // 1. First cycle: stress=50, capacity=10, overstressed=true, consumer.is_active becomes false
        // 2. Second cycle: stress=0 (consumer inactive), capacity=10, overstressed=false, speed=1.0
        // The key result is that the consumer gets deactivated as a protection mechanism
        for _ in 0..5 {
            app.update();
        }

        // Assertions - Check the result of overstress: consumer should be deactivated
        let power_node_source = app.world().get::<PowerNode>(source_entity).expect("PowerNode should be added to source");
        assert!(power_node_source.group_id.is_some(), "Should be in a group");

        let consumer = app.world().get::<PowerConsumer>(consumer_entity).unwrap();

        // The key assertion: consumer should have been deactivated due to overstress
        // This is the protection mechanism - when overstressed, consumers are deactivated
        assert!(!consumer.is_active, "Consumer should be deactivated due to initial overstress");

        // After consumer is deactivated, the network recovers (no more stress demand)
        // So speed should be back to 1.0 (system recovered)
        let source = app.world().get::<PowerSource>(source_entity).unwrap();
        assert_eq!(source.current_speed, 1.0, "Network should recover after consumer deactivation");
    }
}
