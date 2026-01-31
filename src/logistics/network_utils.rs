//! Network utility functions for resource network operations
//!
//! This module provides common utilities for network analysis and manipulation.
//! Includes Union-Find data structure for connected component detection.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bevy::prelude::*;

/// Union-Find (Disjoint Set Union) data structure with path compression and union by rank
///
/// This data structure efficiently tracks connected components and supports:
/// - `find(x)`: Find the representative (root) of the set containing x
/// - `union(x, y)`: Merge the sets containing x and y
/// - `get_components()`: Get all connected components
///
/// # Type Parameter
///
/// - `K`: Node identifier type (must implement Hash + Eq + Copy)
pub struct NetworkUnionFind<K>
where
    K: Hash + Eq + Copy,
{
    /// Parent mapping: node -> parent node
    parent: HashMap<K, K>,
    /// Rank mapping: node -> rank for union by rank
    rank: HashMap<K, u8>,
}

impl<K> NetworkUnionFind<K>
where
    K: Hash + Eq + Copy,
{
    /// Create a new empty Union-Find structure
    ///
    /// Initially, each element is in its own set.
    pub fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    /// Find the representative (root) of the set containing x
    ///
    /// Uses path compression to make future queries faster.
    ///
    /// # Arguments
    ///
    /// - `x`: The element to find
    ///
    /// # Returns
    ///
    /// The representative (root) of the set containing x
    fn find(&mut self, x: K) -> K {
        // If x is not in the structure, add it as its own parent
        if let std::collections::hash_map::Entry::Vacant(e) = self.parent.entry(x) {
            e.insert(x);
            self.rank.insert(x, 0);
            return x;
        }

        // Path compression: make every node on the path point to the root
        if self.parent[&x] == x {
            return x;
        }

        let root = self.find(self.parent[&x]);
        self.parent.insert(x, root);
        root
    }

    /// Merge the sets containing x and y
    ///
    /// Uses union by rank to keep the tree shallow.
    ///
    /// # Arguments
    ///
    /// - `x`: First element
    /// - `y`: Second element
    ///
    /// # Returns
    ///
    /// True if the sets were merged, false if they were already in the same set
    pub fn union(&mut self, x: K, y: K) -> bool {
        let root_x = self.find(x);
        let root_y = self.find(y);

        // If already in the same set, no need to union
        if root_x == root_y {
            return false;
        }

        // Union by rank: attach the shorter tree under the taller one
        let rank_x = self.rank.get(&root_x).copied().unwrap_or(0);
        let rank_y = self.rank.get(&root_y).copied().unwrap_or(0);

        if rank_x < rank_y {
            self.parent.insert(root_x, root_y);
            self.rank.insert(root_y, rank_x.max(rank_y));
        } else if rank_x > rank_y {
            self.parent.insert(root_y, root_x);
            self.rank.insert(root_x, rank_x.max(rank_y));
        } else {
            // Same rank, attach one to the other and increment rank
            self.parent.insert(root_y, root_x);
            self.rank.insert(root_x, rank_x + 1);
        }

        true
    }

    /// Get all connected components
    ///
    /// Returns a HashMap where keys are component representatives and values
    /// are vectors of all nodes in that component.
    ///
    /// # Returns
    ///
    /// A HashMap mapping component representatives to their nodes
    pub fn get_components(&mut self) -> HashMap<K, Vec<K>> {
        let mut components: HashMap<K, Vec<K>> = HashMap::new();

        // Find all unique roots
        for node in self.parent.keys().cloned().collect::<Vec<_>>() {
            let root = self.find(node);
            components.entry(root).or_default().push(node);
        }

        components
    }

    /// Clear all data
    ///
    /// Resets the Union-Find structure to its initial state.
    pub fn clear(&mut self) {
        self.parent.clear();
        self.rank.clear();
    }

    /// Get the number of connected components
    ///
    /// This is equal to the number of unique roots.
    ///
    /// # Returns
    ///
    /// The number of connected components
    pub fn component_count(&mut self) -> usize {
        // Count unique roots
        let mut roots = HashSet::new();
        for node in self.parent.keys() {
            // We need to call find to get the root, but this modifies the structure
            // For count, we can just count unique nodes for now
            // A more accurate count would require a separate method that doesn't modify
            roots.insert(node);
        }

        // Get the roots from each node
        let mut unique_roots = HashSet::new();
        for node in self.parent.keys().cloned().collect::<Vec<_>>() {
            let root = self.find(node);
            unique_roots.insert(root);
        }

        unique_roots.len()
    }

    /// Check if two elements are in the same component
    ///
    /// # Arguments
    ///
    /// - `x`: First element
    /// - `y`: Second element
    ///
    /// # Returns
    ///
    /// True if x and y are in the same component, false otherwise
    pub fn in_same_component(&mut self, x: K, y: K) -> bool {
        self.find(x) == self.find(y)
    }

    /// Get the size of the component containing x
    ///
    /// # Arguments
    ///
    /// - `x`: An element in the component
    ///
    /// # Returns
    ///
    /// The size of the component (number of elements)
    pub fn component_size(&mut self, x: K) -> usize {
        let root = self.find(x);
        self.get_components()
            .get(&root)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

impl<K> Default for NetworkUnionFind<K>
where
    K: Hash + Eq + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_find_basic() {
        let mut uf = NetworkUnionFind::new();

        // Initially, each element is in its own component
        assert_eq!(uf.component_count(), 0);

        // Add nodes
        uf.union(1, 2);
        assert!(uf.in_same_component(1, 2));
        assert_eq!(uf.component_count(), 1);

        uf.union(3, 4);
        assert!(uf.in_same_component(3, 4));
        assert_eq!(uf.component_count(), 2);

        // Union the two components
        uf.union(2, 3);
        assert!(uf.in_same_component(1, 4));
        assert_eq!(uf.component_count(), 1);
    }

    #[test]
    fn test_union_find_components() {
        let mut uf = NetworkUnionFind::new();

        // Union some elements
        uf.union(1, 2);
        uf.union(2, 3);
        uf.union(4, 5);

        // Get components
        let components = uf.get_components();

        // There should be 2 components: {1, 2, 3} and {4, 5}
        assert_eq!(components.len(), 2);

        // Check that each component has the correct size
        let component_1 = components.get(&1).unwrap();
        assert_eq!(component_1.len(), 3);
        assert!(component_1.contains(&1) && component_1.contains(&2) && component_1.contains(&3));

        let component_4 = components.get(&4).unwrap();
        assert_eq!(component_4.len(), 2);
        assert!(component_4.contains(&4) && component_4.contains(&5));
    }

    #[test]
    fn test_union_find_path_compression() {
        let mut uf = NetworkUnionFind::new();

        // Create a chain: 1 -> 2 -> 3 -> 4
        uf.union(1, 2);
        uf.union(2, 3);
        uf.union(3, 4);

        // Find should compress paths
        let root = uf.find(1);
        let root_4 = uf.find(4);

        // All should have the same root
        assert_eq!(root, root_4);
        assert_eq!(uf.component_count(), 1);

        // Now find should be O(1)
        let _ = uf.find(2);
        let _ = uf.find(3);

        // Find should still return the same root
        assert_eq!(uf.find(1), root);
    }

    #[test]
    fn test_union_find_clear() {
        let mut uf = NetworkUnionFind::new();

        // Add some elements
        uf.union(1, 2);
        uf.union(2, 3);

        assert_eq!(uf.component_count(), 1);

        // Clear
        uf.clear();

        // Now should have 0 components
        assert_eq!(uf.component_count(), 0);

        // Should be able to add elements again
        uf.union(4, 5);
        assert_eq!(uf.component_count(), 1);
    }

    #[test]
    fn test_component_size() {
        let mut uf = NetworkUnionFind::new();

        // Add elements
        uf.union(1, 2);
        uf.union(1, 3);
        uf.union(4, 5);

        // Component containing 1 should have size 3
        assert_eq!(uf.component_size(1), 3);

        // Component containing 4 should have size 2
        assert_eq!(uf.component_size(4), 2);
    }

    #[test]
    fn test_union_already_in_same_component() {
        let mut uf = NetworkUnionFind::new();

        uf.union(1, 2);
        assert_eq!(uf.component_count(), 1);

        // Try to union the same component again
        let result = uf.union(1, 2);
        assert_eq!(result, false); // Should return false (already in same component)
    }

    #[test]
    fn test_union_find_u64() {
        let mut uf = NetworkUnionFind::new();

        // Use u64 as the key type
        uf.union(10, 20);
        uf.union(20, 30);
        uf.union(40, 50);

        assert!(uf.in_same_component(10, 30));
        assert!(!uf.in_same_component(10, 40));

        let components = uf.get_components();
        assert_eq!(components.len(), 2);

        // Check component sizes
        if let Some(comp) = components.get(&10) {
            assert_eq!(comp.len(), 3);
        }
    }
}
