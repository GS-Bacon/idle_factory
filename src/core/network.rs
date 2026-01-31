//! Network graph implementation
//!
//! This module provides a generic graph data structure using adjacency maps.
//! The graph is undirected, weighted, and supports various network types.

use bevy::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

/// Type alias for signal networks
///
/// Signal networks are undirected graphs representing signal propagation paths.
/// Uses u64 for node identifiers and u8 for signal values.
pub type SignalNetwork = NetworkGraph<u64, u8>;

/// Type alias for power networks
pub type PowerNetwork = NetworkGraph<u64, f32>;

/// Type alias for fluid networks
pub type FluidNetwork = NetworkGraph<u64, f32>;

/// A generic undirected graph with weighted edges
///
/// Uses HashMap for O(1) neighbor lookups and edge updates.
/// The graph is undirected, meaning edges are stored bidirectionally.
///
/// # Type Parameters
///
/// - `K`: Node identifier type (must implement Hash + Eq + Copy)
/// - `V`: Edge weight type (must implement Default + Copy)
#[derive(Clone, Debug)]
pub struct NetworkGraph<K, V>
where
    K: Hash + Eq + Copy,
    V: Default + Copy,
{
    /// Adjacency map: node -> (neighbor, weight)
    nodes: HashMap<K, Vec<(K, V)>>,
    /// Edge weights for O(1) lookups
    edges: HashMap<(K, K), V>,
    /// Total number of nodes
    count: usize,
}

impl<K, V> NetworkGraph<K, V>
where
    K: Hash + Eq + Copy,
    V: Default + Copy,
{
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            count: 0,
        }
    }

    /// Create a new graph with an initial node
    pub fn with_capacity(initial_size: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(initial_size),
            edges: HashMap::with_capacity(initial_size),
            count: 0,
        }
    }

    /// Add a node to the graph
    ///
    /// Returns Ok(()) if the node was added, or Err(()) if it already exists.
    #[allow(clippy::result_unit_err)]
    pub fn add_node(&mut self, node: K) -> Result<(), ()> {
        if self.nodes.contains_key(&node) {
            return Err(());
        }

        self.nodes.insert(node, Vec::new());
        self.count += 1;
        Ok(())
    }

    /// Remove a node from the graph
    ///
    /// Returns Ok(()) if the node was removed, or Err(()) if it didn't exist.
    #[allow(clippy::result_unit_err)]
    pub fn remove_node(&mut self, node: K) -> Result<(), ()> {
        if !self.nodes.contains_key(&node) {
            return Err(());
        }

        // Remove all edges connected to this node
        let neighbors = self.nodes.remove(&node).unwrap();
        for neighbor in neighbors {
            let other = neighbor.0;
            self.edges.remove(&(node, other));
            self.edges.remove(&(other, node));
            // Remove neighbor reference from the other node
            let neighbor_list = self.nodes.get_mut(&other).unwrap();
            neighbor_list.retain(|(n, _)| n != &node);
        }

        self.count -= 1;
        Ok(())
    }

    /// Connect two nodes with an edge
    ///
    /// Returns Ok(()) if the edge was added, or Err(()) if either node
    /// doesn't exist or the edge already exists.
    #[allow(clippy::result_unit_err)]
    pub fn connect(&mut self, node_a: K, node_b: K, weight: V) -> Result<(), ()> {
        // Check if nodes exist
        if !self.nodes.contains_key(&node_a) {
            return Err(());
        }
        if !self.nodes.contains_key(&node_b) {
            return Err(());
        }

        // Check if edge already exists
        if self.edges.contains_key(&(node_a, node_b)) {
            return Err(());
        }

        // Add edge in both directions (undirected graph)
        self.edges.insert((node_a, node_b), weight);
        self.edges.insert((node_b, node_a), weight);

        // Update adjacency lists
        {
            let node_a_list = self.nodes.get_mut(&node_a).unwrap();
            node_a_list.push((node_b, weight));
        }
        {
            let node_b_list = self.nodes.get_mut(&node_b).unwrap();
            node_b_list.push((node_a, weight));
        }

        Ok(())
    }

    /// Disconnect two nodes
    ///
    /// Returns Ok(()) if the edge was removed, or Err(()) if either node
    /// doesn't exist or the edge doesn't exist.
    #[allow(clippy::result_unit_err)]
    pub fn disconnect(&mut self, node_a: K, node_b: K) -> Result<(), ()> {
        if !self.nodes.contains_key(&node_a) {
            return Err(());
        }
        if !self.nodes.contains_key(&node_b) {
            return Err(());
        }

        let edge_a_b = self.edges.remove(&(node_a, node_b));
        let edge_b_a = self.edges.remove(&(node_b, node_a));

        if edge_a_b.is_none() || edge_b_a.is_none() {
            return Err(());
        }

        // Remove from adjacency lists
        {
            let node_a_list = self.nodes.get_mut(&node_a).unwrap();
            node_a_list.retain(|(n, _)| n != &node_b);
        }
        {
            let node_b_list = self.nodes.get_mut(&node_b).unwrap();
            node_b_list.retain(|(n, _)| n != &node_a);
        }

        Ok(())
    }

    /// Get the weight of an edge
    ///
    /// Returns Some(weight) if the edge exists, None otherwise.
    pub fn get_edge(&self, node_a: K, node_b: K) -> Option<V> {
        self.edges.get(&(node_a, node_b)).copied()
    }

    /// Get the neighbors of a node
    ///
    /// Returns an iterator over (neighbor, weight) pairs.
    pub fn neighbors(&self, node: K) -> impl Iterator<Item = &(K, V)> {
        self.nodes
            .get(&node)
            .map(|list| list.as_slice())
            .unwrap_or(&[])
            .iter()
    }

    /// Get all edges
    pub fn edges(&self) -> impl Iterator<Item = (&(K, K), &V)> {
        self.edges.iter()
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &K> {
        self.nodes.keys()
    }

    /// Clear all nodes and edges
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.count = 0;
    }
}

impl<K, V> Default for NetworkGraph<K, V>
where
    K: Hash + Eq + Copy + Default,
    V: Default + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_add_remove() {
        let mut graph = NetworkGraph::new();

        // Add nodes
        assert!(graph.add_node(1u64).is_ok());
        assert!(graph.add_node(2u64).is_ok());
        assert!(graph.add_node(3u64).is_ok());
        assert_eq!(graph.len(), 3);

        // Try to add duplicate node
        assert!(graph.add_node(1u64).is_err());

        // Remove node
        assert!(graph.remove_node(2u64).is_ok());
        assert_eq!(graph.len(), 2);

        // Try to remove non-existent node
        assert!(graph.remove_node(2u64).is_err());

        // Connect nodes
        assert!(graph.connect(1u64, 3u64, 5u8).is_ok());
        assert_eq!(graph.get_edge(1u64, 3u64), Some(5u8));

        // Try to connect to non-existent node
        assert!(graph.connect(2u64, 3u64, 10u8).is_err());
    }

    #[test]
    fn test_network_neighbors() {
        let mut graph = NetworkGraph::new();

        graph.add_node(1u64).unwrap();
        graph.add_node(2u64).unwrap();
        graph.add_node(3u64).unwrap();

        graph.connect(1u64, 2u64, 1u8).unwrap();
        graph.connect(1u64, 3u64, 2u8).unwrap();

        let neighbors = graph.neighbors(1u64).collect::<Vec<_>>();
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_network_disconnect() {
        let mut graph = NetworkGraph::new();

        graph.add_node(1u64).unwrap();
        graph.add_node(2u64).unwrap();

        graph.connect(1u64, 2u64, 1u8).unwrap();
        assert_eq!(graph.get_edge(1u64, 2u64), Some(1u8));

        graph.disconnect(1u64, 2u64).unwrap();
        assert_eq!(graph.get_edge(1u64, 2u64), None);
    }
}

#[cfg(test)]
mod signal_tests {
    use super::*;
    use crate::core::SignalNetwork;

    #[test]
    fn test_signal_network_add() {
        let mut network = SignalNetwork::new();

        // Add nodes
        assert!(network.add_node(1u64).is_ok());
        assert!(network.add_node(2u64).is_ok());
        assert!(network.add_node(3u64).is_ok());
        assert_eq!(network.len(), 3);

        // Verify node was added
        assert!(network.nodes().any(|&n| n == 1u64));
        assert!(network.nodes().any(|&n| n == 2u64));
        assert!(network.nodes().any(|&n| n == 3u64));

        // Verify neighbors count
        let neighbors = network.neighbors(1u64).count();
        assert_eq!(neighbors, 0);

        // Connect nodes
        assert!(network.connect(1u64, 2u64, 1u8).is_ok());
        assert!(network.connect(2u64, 3u64, 2u8).is_ok());

        // Verify neighbors
        let neighbors_1 = network.neighbors(1u64).collect::<Vec<_>>();
        assert_eq!(neighbors_1.len(), 1);
        assert_eq!(neighbors_1[0].0, 2u64);
        assert_eq!(neighbors_1[0].1, 1u8);

        let neighbors_2 = network.neighbors(2u64).collect::<Vec<_>>();
        assert_eq!(neighbors_2.len(), 2);
    }

    #[test]
    fn test_signal_network_connect() {
        let mut network = SignalNetwork::new();

        // Add nodes and connect
        network.add_node(1u64).unwrap();
        network.add_node(2u64).unwrap();

        assert!(network.connect(1u64, 2u64, 5u8).is_ok());

        // Verify bidirectional connection
        let neighbors_1 = network.neighbors(1u64).collect::<Vec<_>>();
        assert_eq!(neighbors_1.len(), 1);
        assert_eq!(neighbors_1[0].0, 2u64);

        let neighbors_2 = network.neighbors(2u64).collect::<Vec<_>>();
        assert_eq!(neighbors_2.len(), 1);
        assert_eq!(neighbors_2[0].0, 1u64);

        // Verify edge weight
        assert_eq!(network.get_edge(1u64, 2u64), Some(5u8));
        assert_eq!(network.get_edge(2u64, 1u64), Some(5u8));

        // Try to connect duplicate edge
        assert!(network.connect(1u64, 2u64, 10u8).is_err());
    }
}
