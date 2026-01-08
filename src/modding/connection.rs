//! Connection management for Mod API WebSocket server
//!
//! Tracks connected mods and their subscribed events.

use std::collections::HashMap;

/// Mod connection state
#[derive(Debug, Clone)]
pub struct ModConnection {
    /// Unique connection ID
    pub id: u64,
    /// Mod name (set after handshake)
    pub mod_name: Option<String>,
    /// Mod version (set after handshake)
    pub mod_version: Option<String>,
    /// Events this connection is subscribed to
    pub subscribed_events: Vec<String>,
    /// Connection timestamp (Unix epoch seconds)
    pub connected_at: u64,
}

impl ModConnection {
    /// Create a new connection
    pub fn new(id: u64) -> Self {
        Self {
            id,
            mod_name: None,
            mod_version: None,
            subscribed_events: Vec::new(),
            connected_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Check if this connection has identified itself
    pub fn is_identified(&self) -> bool {
        self.mod_name.is_some()
    }

    /// Subscribe to an event type
    pub fn subscribe(&mut self, event_type: &str) -> bool {
        if !self.subscribed_events.contains(&event_type.to_string()) {
            self.subscribed_events.push(event_type.to_string());
            true
        } else {
            false
        }
    }

    /// Unsubscribe from an event type
    pub fn unsubscribe(&mut self, event_type: &str) -> bool {
        if let Some(pos) = self.subscribed_events.iter().position(|e| e == event_type) {
            self.subscribed_events.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if subscribed to an event type
    pub fn is_subscribed(&self, event_type: &str) -> bool {
        self.subscribed_events.contains(&event_type.to_string())
    }
}

/// Connection manager for tracking all mod connections
#[derive(Debug, Default)]
pub struct ConnectionManager {
    /// Active connections (id -> connection)
    connections: HashMap<u64, ModConnection>,
    /// Next connection ID
    next_id: u64,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new connection, returns the connection ID
    pub fn add_connection(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id, ModConnection::new(id));
        id
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, id: u64) -> Option<ModConnection> {
        self.connections.remove(&id)
    }

    /// Get a connection by ID
    pub fn get(&self, id: u64) -> Option<&ModConnection> {
        self.connections.get(&id)
    }

    /// Get a mutable connection by ID
    pub fn get_mut(&mut self, id: u64) -> Option<&mut ModConnection> {
        self.connections.get_mut(&id)
    }

    /// Get all connection IDs subscribed to an event type
    pub fn subscribers(&self, event_type: &str) -> Vec<u64> {
        self.connections
            .values()
            .filter(|c| c.is_subscribed(event_type))
            .map(|c| c.id)
            .collect()
    }

    /// Get all connection IDs
    pub fn all_ids(&self) -> Vec<u64> {
        self.connections.keys().copied().collect()
    }

    /// Get connection count
    pub fn count(&self) -> usize {
        self.connections.len()
    }

    /// Check if a connection exists
    pub fn exists(&self, id: u64) -> bool {
        self.connections.contains_key(&id)
    }

    /// Get all identified connections (those that completed handshake)
    pub fn identified(&self) -> impl Iterator<Item = &ModConnection> {
        self.connections.values().filter(|c| c.is_identified())
    }

    /// Identify a connection (set mod info after handshake)
    pub fn identify(&mut self, id: u64, mod_name: &str, mod_version: &str) -> bool {
        if let Some(conn) = self.connections.get_mut(&id) {
            conn.mod_name = Some(mod_name.to_string());
            conn.mod_version = Some(mod_version.to_string());
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_connection_new() {
        let conn = ModConnection::new(1);

        assert_eq!(conn.id, 1);
        assert!(conn.mod_name.is_none());
        assert!(!conn.is_identified());
        assert!(conn.subscribed_events.is_empty());
    }

    #[test]
    fn test_mod_connection_subscribe() {
        let mut conn = ModConnection::new(1);

        // First subscribe succeeds
        assert!(conn.subscribe("tick"));
        assert!(conn.is_subscribed("tick"));

        // Duplicate subscribe returns false
        assert!(!conn.subscribe("tick"));

        // Subscribe to another event
        assert!(conn.subscribe("item_crafted"));
        assert_eq!(conn.subscribed_events.len(), 2);
    }

    #[test]
    fn test_mod_connection_unsubscribe() {
        let mut conn = ModConnection::new(1);
        conn.subscribe("tick");
        conn.subscribe("item_crafted");

        // Unsubscribe succeeds
        assert!(conn.unsubscribe("tick"));
        assert!(!conn.is_subscribed("tick"));

        // Unsubscribe from non-subscribed event returns false
        assert!(!conn.unsubscribe("tick"));
    }

    #[test]
    fn test_connection_manager_add() {
        let mut manager = ConnectionManager::new();

        let id1 = manager.add_connection();
        let id2 = manager.add_connection();

        assert_ne!(id1, id2);
        assert_eq!(manager.count(), 2);
        assert!(manager.exists(id1));
        assert!(manager.exists(id2));
    }

    #[test]
    fn test_connection_manager_remove() {
        let mut manager = ConnectionManager::new();
        let id = manager.add_connection();

        let removed = manager.remove_connection(id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, id);
        assert!(!manager.exists(id));
    }

    #[test]
    fn test_connection_manager_get() {
        let mut manager = ConnectionManager::new();
        let id = manager.add_connection();

        assert!(manager.get(id).is_some());
        assert!(manager.get(999).is_none());

        // Mutable access
        let conn = manager.get_mut(id).unwrap();
        conn.subscribe("test");

        // Verify change persisted
        assert!(manager.get(id).unwrap().is_subscribed("test"));
    }

    #[test]
    fn test_connection_manager_subscribers() {
        let mut manager = ConnectionManager::new();
        let id1 = manager.add_connection();
        let id2 = manager.add_connection();
        let id3 = manager.add_connection();

        manager.get_mut(id1).unwrap().subscribe("tick");
        manager.get_mut(id2).unwrap().subscribe("tick");
        manager.get_mut(id3).unwrap().subscribe("item_crafted");

        let tick_subs = manager.subscribers("tick");
        assert_eq!(tick_subs.len(), 2);
        assert!(tick_subs.contains(&id1));
        assert!(tick_subs.contains(&id2));

        let item_subs = manager.subscribers("item_crafted");
        assert_eq!(item_subs.len(), 1);
        assert!(item_subs.contains(&id3));
    }

    #[test]
    fn test_connection_manager_identify() {
        let mut manager = ConnectionManager::new();
        let id = manager.add_connection();

        assert!(!manager.get(id).unwrap().is_identified());

        assert!(manager.identify(id, "test_mod", "1.0.0"));

        let conn = manager.get(id).unwrap();
        assert!(conn.is_identified());
        assert_eq!(conn.mod_name, Some("test_mod".to_string()));
        assert_eq!(conn.mod_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_connection_manager_identified() {
        let mut manager = ConnectionManager::new();
        let id1 = manager.add_connection();
        let _id2 = manager.add_connection();

        manager.identify(id1, "mod1", "1.0.0");
        // _id2 not identified

        let identified: Vec<_> = manager.identified().collect();
        assert_eq!(identified.len(), 1);
        assert_eq!(identified[0].id, id1);
    }

    #[test]
    fn test_connection_manager_all_ids() {
        let mut manager = ConnectionManager::new();
        let id1 = manager.add_connection();
        let id2 = manager.add_connection();

        let ids = manager.all_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }
}
