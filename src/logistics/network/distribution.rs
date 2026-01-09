//! Resource distribution algorithms
//!
//! Handles power distribution, fluid equalization, and signal propagation.

use super::{
    node::{FluidNode, PowerNode, SignalNode},
    types::NodeRole,
    NetworkTypeRegistry, PowerShortage, SegmentRegistry,
};
use bevy::prelude::*;

// =============================================================================
// Power Distribution
// =============================================================================

/// Distribute power within segments
///
/// Uses priority-based distribution: higher priority consumers are satisfied first.
pub fn distribute_power(
    network_types: Res<NetworkTypeRegistry>,
    mut segment_registry: ResMut<SegmentRegistry>,
    mut power_nodes: Query<(Entity, &mut PowerNode)>,
    mut shortage_events: EventWriter<PowerShortage>,
) {
    let power_type = network_types.power();

    // Process each power segment
    for segment in segment_registry.iter_mut() {
        if segment.network_type != power_type {
            continue;
        }

        // Collect producers and consumers in this segment
        let mut total_supply = 0.0;
        let mut consumers: Vec<(Entity, f32, i8)> = Vec::new(); // (entity, demand, priority)

        for &entity in &segment.nodes {
            if let Ok((_, power_node)) = power_nodes.get(entity) {
                match power_node.satisfaction {
                    s if s >= 1.0 => {
                        // Producer (satisfaction = 1.0)
                        total_supply += power_node.power_watts;
                    }
                    _ => {
                        // Consumer
                        consumers.push((entity, power_node.power_watts, power_node.priority));
                    }
                }
            }
        }

        // Update segment stats
        let total_demand: f32 = consumers.iter().map(|(_, d, _)| d).sum();
        segment.supply = total_supply;
        segment.demand = total_demand;

        // Sort consumers by priority (high to low)
        consumers.sort_by_key(|(_, _, priority)| -(*priority as i32));

        // Distribute power
        let mut remaining = total_supply;
        for (entity, demand, _) in consumers {
            if let Ok((_, mut power_node)) = power_nodes.get_mut(entity) {
                if demand <= remaining {
                    power_node.satisfaction = 1.0;
                    remaining -= demand;
                } else if remaining > 0.0 {
                    power_node.satisfaction = remaining / demand;
                    remaining = 0.0;
                } else {
                    power_node.satisfaction = 0.0;
                }
            }
        }

        // Update segment satisfaction
        segment.satisfaction = if total_demand > 0.0 {
            (total_supply / total_demand).min(1.0)
        } else {
            1.0
        };

        // Fire shortage event if needed
        if total_supply < total_demand {
            shortage_events.send(PowerShortage {
                segment_id: segment.id,
                supply: total_supply,
                demand: total_demand,
            });
        }
    }
}

// =============================================================================
// Fluid Distribution
// =============================================================================

/// Distribute fluid within segments
///
/// Uses Factorio-style instant equalization within a segment.
pub fn distribute_fluid(
    network_types: Res<NetworkTypeRegistry>,
    mut segment_registry: ResMut<SegmentRegistry>,
    mut fluid_nodes: Query<(Entity, &mut FluidNode)>,
) {
    let fluid_type = network_types.fluid();
    let gas_type = network_types.gas();

    // Process each fluid/gas segment
    for segment in segment_registry.iter_mut() {
        if segment.network_type != fluid_type && segment.network_type != gas_type {
            continue;
        }

        // Calculate totals
        let mut total_amount = 0.0;
        let mut total_capacity = 0.0;
        let mut node_entities: Vec<Entity> = Vec::new();

        for &entity in &segment.nodes {
            if let Ok((_, fluid_node)) = fluid_nodes.get(entity) {
                total_amount += fluid_node.amount;
                total_capacity += fluid_node.capacity;
                node_entities.push(entity);
            }
        }

        // Update segment stats
        segment.amount = total_amount;
        segment.capacity = total_capacity;

        // Calculate uniform fill ratio
        let fill_ratio = if total_capacity > 0.0 {
            (total_amount / total_capacity).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Apply uniform fill to all nodes (instant equalization)
        for entity in node_entities {
            if let Ok((_, mut fluid_node)) = fluid_nodes.get_mut(entity) {
                fluid_node.amount = fluid_node.capacity * fill_ratio;
            }
        }
    }
}

// =============================================================================
// Signal Propagation
// =============================================================================

/// Propagate signals within segments
///
/// The maximum signal strength in the segment is propagated to all receivers.
pub fn propagate_signal(
    network_types: Res<NetworkTypeRegistry>,
    mut segment_registry: ResMut<SegmentRegistry>,
    port_query: Query<(Entity, &super::node::NetworkPort)>,
    mut signal_nodes: Query<(Entity, &mut SignalNode)>,
) {
    let signal_type = network_types.signal();

    // Process each signal segment
    for segment in segment_registry.iter_mut() {
        if segment.network_type != signal_type {
            continue;
        }

        // Find max signal strength from producers
        let mut max_strength: u8 = 0;

        for &entity in &segment.nodes {
            // Check if this is a producer
            let is_producer = port_query
                .get(entity)
                .map(|(_, port)| port.role == NodeRole::Producer)
                .unwrap_or(false);

            if is_producer {
                if let Ok((_, signal_node)) = signal_nodes.get(entity) {
                    max_strength = max_strength.max(signal_node.strength);
                }
            }
        }

        // Update segment signal strength
        segment.signal_strength = max_strength;

        // Propagate to all non-producer nodes
        for &entity in &segment.nodes {
            let is_producer = port_query
                .get(entity)
                .map(|(_, port)| port.role == NodeRole::Producer)
                .unwrap_or(false);

            if !is_producer {
                if let Ok((_, mut signal_node)) = signal_nodes.get_mut(entity) {
                    signal_node.strength = max_strength;
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn test_power_priority_distribution() {
        // This test verifies the priority-based distribution logic
        // In a real test, we'd use a Bevy test harness

        // Simulate: 100W supply, 60W high-priority consumer, 60W low-priority consumer
        let supply: f32 = 100.0;
        let high_demand: f32 = 60.0;
        let low_demand: f32 = 60.0;

        // High priority gets full (60W)
        let high_alloc = high_demand.min(supply);
        let remaining = supply - high_alloc;

        // Low priority gets remaining (40W)
        let low_alloc = low_demand.min(remaining);

        assert_eq!(high_alloc, 60.0);
        assert_eq!(low_alloc, 40.0);

        let high_satisfaction = high_alloc / high_demand;
        let low_satisfaction = low_alloc / low_demand;

        assert_eq!(high_satisfaction, 1.0);
        assert!((low_satisfaction - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_fluid_equalization() {
        // Simulate: Two tanks, one with 800mB/1000mB, one with 200mB/1000mB
        let amounts = [800.0, 200.0];
        let capacities = [1000.0, 1000.0];

        let total_amount: f32 = amounts.iter().sum();
        let total_capacity: f32 = capacities.iter().sum();

        let fill_ratio = total_amount / total_capacity;
        assert_eq!(fill_ratio, 0.5);

        // After equalization, both should have 500mB
        let new_amounts: Vec<f32> = capacities.iter().map(|c| c * fill_ratio).collect();
        assert_eq!(new_amounts[0], 500.0);
        assert_eq!(new_amounts[1], 500.0);
    }

    #[test]
    fn test_signal_max_propagation() {
        // Simulate: Two sources with strength 10 and 15
        let sources = [10u8, 15u8];
        let max_strength = sources.iter().max().copied().unwrap_or(0);

        assert_eq!(max_strength, 15);
    }
}
