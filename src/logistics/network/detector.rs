//! Segment detection system
//!
//! Detects and manages network segments using flood fill algorithm.

use super::{
    conduit::{Pipe, SignalWire, Wire},
    node::NetworkPort,
    registry::SegmentRegistry,
    types::SegmentId,
    virtual_link::VirtualLinkRegistry,
    NetworkTypeId, NetworkTypeRegistry, SegmentBroken, SegmentFormed,
};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

// =============================================================================
// Block Change Events (trigger segment detection)
// =============================================================================

/// Event fired when a network-relevant block is placed
#[derive(Event, Debug)]
pub struct NetworkBlockPlaced {
    pub position: IVec3,
    pub entity: Entity,
    pub network_type: NetworkTypeId,
}

/// Event fired when a network-relevant block is removed
#[derive(Event, Debug)]
pub struct NetworkBlockRemoved {
    pub position: IVec3,
    pub network_type: NetworkTypeId,
}

// =============================================================================
// Segment Detection System
// =============================================================================

/// Main segment detection system
///
/// Handles segment creation, merging, and splitting based on block changes.
#[allow(clippy::too_many_arguments)]
pub fn detect_segments(
    mut commands: Commands,
    mut placed_events: EventReader<NetworkBlockPlaced>,
    mut removed_events: EventReader<NetworkBlockRemoved>,
    mut segment_registry: ResMut<SegmentRegistry>,
    mut segment_formed_events: EventWriter<SegmentFormed>,
    mut segment_broken_events: EventWriter<SegmentBroken>,
    network_types: Res<NetworkTypeRegistry>,
    virtual_links: Res<VirtualLinkRegistry>,
    wire_query: Query<(Entity, &Wire)>,
    pipe_query: Query<(Entity, &Pipe)>,
    signal_wire_query: Query<(Entity, &SignalWire)>,
    port_query: Query<(Entity, &NetworkPort, &Transform)>,
) {
    // Handle block placements
    for event in placed_events.read() {
        handle_block_placed(
            &mut commands,
            event,
            &mut segment_registry,
            &mut segment_formed_events,
            &network_types,
            &virtual_links,
            &wire_query,
            &pipe_query,
            &signal_wire_query,
            &port_query,
        );
    }

    // Handle block removals
    for event in removed_events.read() {
        handle_block_removed(
            event,
            &mut segment_registry,
            &mut segment_broken_events,
            &network_types,
            &virtual_links,
            &wire_query,
            &pipe_query,
            &signal_wire_query,
            &port_query,
        );
    }
}

/// Handle a block being placed
#[allow(clippy::too_many_arguments)]
fn handle_block_placed(
    _commands: &mut Commands,
    event: &NetworkBlockPlaced,
    segment_registry: &mut SegmentRegistry,
    segment_formed_events: &mut EventWriter<SegmentFormed>,
    network_types: &NetworkTypeRegistry,
    virtual_links: &VirtualLinkRegistry,
    wire_query: &Query<(Entity, &Wire)>,
    pipe_query: &Query<(Entity, &Pipe)>,
    signal_wire_query: &Query<(Entity, &SignalWire)>,
    port_query: &Query<(Entity, &NetworkPort, &Transform)>,
) {
    let pos = event.position;
    let network_type = event.network_type;

    // Find adjacent segments
    let adjacent_segments = find_adjacent_segments(
        pos,
        network_type,
        segment_registry,
        network_types,
        virtual_links,
        wire_query,
        pipe_query,
        signal_wire_query,
        port_query,
    );

    match adjacent_segments.len() {
        0 => {
            // No adjacent segments - create new
            let segment_id = segment_registry.create_segment(network_type);
            if let Some(segment) = segment_registry.get_mut(segment_id) {
                segment.add_node(event.entity, pos);
            }
            segment_formed_events.send(SegmentFormed {
                segment_id,
                network_type,
            });
        }
        1 => {
            // One adjacent segment - join it
            let segment_id = adjacent_segments[0];
            if let Some(segment) = segment_registry.get_mut(segment_id) {
                segment.add_node(event.entity, pos);
            }
        }
        _ => {
            // Multiple adjacent segments - merge them
            let first_id = adjacent_segments[0];
            for &other_id in &adjacent_segments[1..] {
                segment_registry.merge_segments(first_id, other_id);
            }
            // Add the new node to the merged segment
            if let Some(segment) = segment_registry.get_mut(first_id) {
                segment.add_node(event.entity, pos);
            }
        }
    }
}

/// Handle a block being removed
#[allow(clippy::too_many_arguments)]
fn handle_block_removed(
    event: &NetworkBlockRemoved,
    segment_registry: &mut SegmentRegistry,
    segment_broken_events: &mut EventWriter<SegmentBroken>,
    network_types: &NetworkTypeRegistry,
    virtual_links: &VirtualLinkRegistry,
    wire_query: &Query<(Entity, &Wire)>,
    pipe_query: &Query<(Entity, &Pipe)>,
    signal_wire_query: &Query<(Entity, &SignalWire)>,
    port_query: &Query<(Entity, &NetworkPort, &Transform)>,
) {
    let pos = event.position;
    let network_type = event.network_type;

    // Find the segment containing this position
    let segment_id = segment_registry
        .iter()
        .find(|s| s.network_type == network_type && s.contains_position(pos))
        .map(|s| s.id);

    let Some(segment_id) = segment_id else {
        return;
    };

    // Remove the node from the segment
    let remaining_nodes: Vec<(Entity, IVec3)> = {
        if let Some(segment) = segment_registry.get_mut(segment_id) {
            // Remove the node
            if let Some(&entity) = segment.node_positions.get(&pos) {
                segment.remove_node(entity, pos);
            }

            // Get remaining nodes
            segment
                .node_positions
                .iter()
                .map(|(&p, &e)| (e, p))
                .collect()
        } else {
            return;
        }
    };

    // If segment is now empty, just remove it
    if remaining_nodes.is_empty() {
        segment_registry.remove(segment_id);
        segment_broken_events.send(SegmentBroken {
            segment_id,
            new_segments: vec![],
        });
        return;
    }

    // Check if segment is still connected (might need to split)
    let groups = find_connected_groups(
        &remaining_nodes,
        network_type,
        network_types,
        virtual_links,
        wire_query,
        pipe_query,
        signal_wire_query,
        port_query,
    );

    if groups.len() > 1 {
        // Need to split the segment
        let new_ids = segment_registry.split_segment(segment_id, groups);
        segment_broken_events.send(SegmentBroken {
            segment_id,
            new_segments: new_ids,
        });
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Find segments adjacent to a position
#[allow(clippy::too_many_arguments)]
fn find_adjacent_segments(
    pos: IVec3,
    network_type: NetworkTypeId,
    segment_registry: &SegmentRegistry,
    network_types: &NetworkTypeRegistry,
    virtual_links: &VirtualLinkRegistry,
    wire_query: &Query<(Entity, &Wire)>,
    pipe_query: &Query<(Entity, &Pipe)>,
    signal_wire_query: &Query<(Entity, &SignalWire)>,
    port_query: &Query<(Entity, &NetworkPort, &Transform)>,
) -> Vec<SegmentId> {
    let mut found_segments = HashSet::new();

    // Check 6 neighbors
    for neighbor_pos in get_neighbors(pos) {
        // Check if neighbor has a matching network node
        if has_network_node_at(
            neighbor_pos,
            network_type,
            network_types,
            wire_query,
            pipe_query,
            signal_wire_query,
            port_query,
        ) {
            // Find which segment this neighbor belongs to
            for segment in segment_registry.iter() {
                if segment.network_type == network_type && segment.contains_position(neighbor_pos) {
                    found_segments.insert(segment.id);
                }
            }
        }
    }

    // Check virtual links
    for link in virtual_links.get_links_from(pos) {
        if link.network_type == network_type {
            let other_pos = if link.from_pos == pos {
                link.to_pos
            } else {
                link.from_pos
            };
            for segment in segment_registry.iter() {
                if segment.network_type == network_type && segment.contains_position(other_pos) {
                    found_segments.insert(segment.id);
                }
            }
        }
    }

    found_segments.into_iter().collect()
}

/// Check if there's a network node at a position
fn has_network_node_at(
    pos: IVec3,
    network_type: NetworkTypeId,
    _network_types: &NetworkTypeRegistry,
    wire_query: &Query<(Entity, &Wire)>,
    pipe_query: &Query<(Entity, &Pipe)>,
    signal_wire_query: &Query<(Entity, &SignalWire)>,
    port_query: &Query<(Entity, &NetworkPort, &Transform)>,
) -> bool {
    // Check wires
    for (_, wire) in wire_query.iter() {
        if wire.position == pos {
            return true;
        }
    }

    // Check pipes
    for (_, pipe) in pipe_query.iter() {
        if pipe.position == pos && pipe.network_type == network_type {
            return true;
        }
    }

    // Check signal wires
    for (_, signal_wire) in signal_wire_query.iter() {
        if signal_wire.position == pos {
            return true;
        }
    }

    // Check network ports
    for (_, port, transform) in port_query.iter() {
        let port_pos = transform.translation.as_ivec3();
        if port_pos == pos && port.network_type == network_type {
            return true;
        }
    }

    false
}

/// Find connected groups of nodes using flood fill
#[allow(clippy::too_many_arguments)]
fn find_connected_groups(
    nodes: &[(Entity, IVec3)],
    network_type: NetworkTypeId,
    network_types: &NetworkTypeRegistry,
    virtual_links: &VirtualLinkRegistry,
    wire_query: &Query<(Entity, &Wire)>,
    pipe_query: &Query<(Entity, &Pipe)>,
    signal_wire_query: &Query<(Entity, &SignalWire)>,
    port_query: &Query<(Entity, &NetworkPort, &Transform)>,
) -> Vec<Vec<(Entity, IVec3)>> {
    if nodes.is_empty() {
        return vec![];
    }

    let node_map: HashMap<IVec3, Entity> = nodes.iter().map(|&(e, p)| (p, e)).collect();
    let mut visited: HashSet<IVec3> = HashSet::new();
    let mut groups: Vec<Vec<(Entity, IVec3)>> = Vec::new();

    for &(_entity, pos) in nodes {
        if visited.contains(&pos) {
            continue;
        }

        // BFS flood fill from this node
        let mut group = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(pos);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(&e) = node_map.get(&current) {
                group.push((e, current));

                // Check neighbors
                for neighbor in get_neighbors(current) {
                    if !visited.contains(&neighbor)
                        && node_map.contains_key(&neighbor)
                        && has_network_node_at(
                            neighbor,
                            network_type,
                            network_types,
                            wire_query,
                            pipe_query,
                            signal_wire_query,
                            port_query,
                        )
                    {
                        queue.push_back(neighbor);
                    }
                }

                // Check virtual links
                for linked_pos in virtual_links.get_connected_positions(current, network_type) {
                    if !visited.contains(&linked_pos) && node_map.contains_key(&linked_pos) {
                        queue.push_back(linked_pos);
                    }
                }
            }
        }

        if !group.is_empty() {
            groups.push(group);
        }
    }

    groups
}

/// Get the 6 neighbors of a position
fn get_neighbors(pos: IVec3) -> [IVec3; 6] {
    [
        pos + IVec3::X,
        pos - IVec3::X,
        pos + IVec3::Y,
        pos - IVec3::Y,
        pos + IVec3::Z,
        pos - IVec3::Z,
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_neighbors() {
        let pos = IVec3::new(5, 5, 5);
        let neighbors = get_neighbors(pos);

        assert!(neighbors.contains(&IVec3::new(6, 5, 5)));
        assert!(neighbors.contains(&IVec3::new(4, 5, 5)));
        assert!(neighbors.contains(&IVec3::new(5, 6, 5)));
        assert!(neighbors.contains(&IVec3::new(5, 4, 5)));
        assert!(neighbors.contains(&IVec3::new(5, 5, 6)));
        assert!(neighbors.contains(&IVec3::new(5, 5, 4)));
    }
}
