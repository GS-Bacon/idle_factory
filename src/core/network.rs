//! Generic network graph structure (Bevy-independent)
//!
//! This module provides a generic graph structure for networks
//! like power grids or fluid pipes.

use std::collections::HashMap;
use std::hash::Hash;

/// A generic network graph for resource distribution
#[derive(Debug, Default)]
pub struct NetworkGraph<K: Eq + Hash + Copy, V: Copy + Default> {
    /// Nodes in the network
    nodes: HashMap<K, NetworkNode<V>>,
    /// Edges (connections) between nodes
    edges: Vec<(K, K)>,
}

/// A node in the network
#[derive(Debug, Clone, Copy)]
pub struct NetworkNode<V: Copy + Default> {
    /// Maximum capacity this node can hold/transfer
    pub capacity: V,
    /// Current amount in this node
    pub current: V,
    /// Whether this node produces (positive) or consumes (negative)
    pub production: V,
}

impl<V: Copy + Default> Default for NetworkNode<V> {
    fn default() -> Self {
        Self {
            capacity: V::default(),
            current: V::default(),
            production: V::default(),
        }
    }
}

impl<K: Eq + Hash + Copy, V: Copy + Default> NetworkGraph<K, V> {
    /// Create a new empty network
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the network
    pub fn add_node(&mut self, key: K, node: NetworkNode<V>) {
        self.nodes.insert(key, node);
    }

    /// Remove a node from the network (also removes connected edges)
    pub fn remove_node(&mut self, key: K) {
        self.nodes.remove(&key);
        self.edges.retain(|(a, b)| *a != key && *b != key);
    }

    /// Connect two nodes
    pub fn connect(&mut self, from: K, to: K) {
        if self.nodes.contains_key(&from) && self.nodes.contains_key(&to) {
            // Avoid duplicate edges
            if !self.edges.contains(&(from, to)) && !self.edges.contains(&(to, from)) {
                self.edges.push((from, to));
            }
        }
    }

    /// Disconnect two nodes
    pub fn disconnect(&mut self, from: K, to: K) {
        self.edges
            .retain(|(a, b)| !((*a == from && *b == to) || (*a == to && *b == from)));
    }

    /// Get a node by key
    pub fn get_node(&self, key: K) -> Option<&NetworkNode<V>> {
        self.nodes.get(&key)
    }

    /// Get a mutable node by key
    pub fn get_node_mut(&mut self, key: K) -> Option<&mut NetworkNode<V>> {
        self.nodes.get_mut(&key)
    }

    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = (&K, &NetworkNode<V>)> {
        self.nodes.iter()
    }

    /// Get all edges
    pub fn edges(&self) -> &[(K, K)] {
        &self.edges
    }

    /// Get neighbors of a node
    pub fn neighbors(&self, key: K) -> impl Iterator<Item = K> + '_ {
        self.edges.iter().filter_map(move |(a, b)| {
            if *a == key {
                Some(*b)
            } else if *b == key {
                Some(*a)
            } else {
                None
            }
        })
    }

    /// Check if the network is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

/// Power-specific network type
pub type PowerNetwork = NetworkGraph<u64, f32>;

/// Fluid-specific network type
pub type FluidNetwork = NetworkGraph<u64, f32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_add_remove() {
        let mut network: NetworkGraph<u64, f32> = NetworkGraph::new();

        network.add_node(
            1,
            NetworkNode {
                capacity: 100.0,
                current: 50.0,
                production: 10.0,
            },
        );
        network.add_node(
            2,
            NetworkNode {
                capacity: 100.0,
                current: 0.0,
                production: -5.0,
            },
        );

        assert_eq!(network.len(), 2);

        network.remove_node(1);
        assert_eq!(network.len(), 1);
    }

    #[test]
    fn test_network_connect() {
        let mut network: NetworkGraph<u64, f32> = NetworkGraph::new();

        network.add_node(1, NetworkNode::default());
        network.add_node(2, NetworkNode::default());
        network.add_node(3, NetworkNode::default());

        network.connect(1, 2);
        network.connect(2, 3);

        let neighbors: Vec<_> = network.neighbors(2).collect();
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&1));
        assert!(neighbors.contains(&3));
    }

    #[test]
    fn test_network_disconnect() {
        let mut network: NetworkGraph<u64, f32> = NetworkGraph::new();

        network.add_node(1, NetworkNode::default());
        network.add_node(2, NetworkNode::default());
        network.connect(1, 2);

        assert_eq!(network.edges().len(), 1);

        network.disconnect(1, 2);
        assert_eq!(network.edges().len(), 0);
    }
}
