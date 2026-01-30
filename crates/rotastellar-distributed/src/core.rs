//! Core types for Earth-space distributed compute coordination.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of compute node in the Earth-space infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// Ground-based node
    Ground,
    /// Orbital node
    Orbital,
}

/// Configuration for a compute node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique identifier for the node
    pub node_id: String,
    /// Type of node (ground or orbital)
    pub node_type: NodeType,
    /// Compute capacity in TFLOPS
    pub compute_tflops: f64,
    /// Memory capacity in GB
    pub memory_gb: f64,
    /// Network bandwidth in Mbps
    pub bandwidth_mbps: f64,
    /// Orbital altitude (for orbital nodes) in km
    pub orbit_altitude_km: Option<f64>,
    /// Ground location (lat, lon) for ground nodes
    pub location: Option<(f64, f64)>,
}

impl NodeConfig {
    /// Create an orbital node configuration.
    pub fn orbital(node_id: &str, altitude_km: f64, compute_tflops: f64) -> Self {
        Self {
            node_id: node_id.to_string(),
            node_type: NodeType::Orbital,
            compute_tflops,
            memory_gb: 32.0,
            bandwidth_mbps: 100.0,
            orbit_altitude_km: Some(altitude_km),
            location: None,
        }
    }

    /// Create a ground node configuration.
    pub fn ground(node_id: &str, lat: f64, lon: f64, compute_tflops: f64) -> Self {
        Self {
            node_id: node_id.to_string(),
            node_type: NodeType::Ground,
            compute_tflops,
            memory_gb: 256.0,
            bandwidth_mbps: 1000.0,
            orbit_altitude_km: None,
            location: Some((lat, lon)),
        }
    }
}

/// Topology of Earth-space compute infrastructure.
#[derive(Debug, Clone, Default)]
pub struct Topology {
    nodes: HashMap<String, NodeConfig>,
    connections: Vec<(String, String, f64)>, // (node1, node2, bandwidth)
}

impl Topology {
    /// Create a new empty topology.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the topology.
    pub fn add_node(&mut self, node: NodeConfig) {
        self.nodes.insert(node.node_id.clone(), node);
    }

    /// Remove a node from the topology.
    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
        self.connections.retain(|(n1, n2, _)| n1 != node_id && n2 != node_id);
    }

    /// Add a connection between two nodes.
    pub fn add_connection(&mut self, node1_id: &str, node2_id: &str, bandwidth_mbps: f64) -> Result<(), &'static str> {
        if !self.nodes.contains_key(node1_id) || !self.nodes.contains_key(node2_id) {
            return Err("Both nodes must exist in topology");
        }
        self.connections.push((node1_id.to_string(), node2_id.to_string(), bandwidth_mbps));
        Ok(())
    }

    /// Get a node by ID.
    pub fn get_node(&self, node_id: &str) -> Option<&NodeConfig> {
        self.nodes.get(node_id)
    }

    /// Get all ground nodes.
    pub fn ground_nodes(&self) -> Vec<&NodeConfig> {
        self.nodes.values().filter(|n| n.node_type == NodeType::Ground).collect()
    }

    /// Get all orbital nodes.
    pub fn orbital_nodes(&self) -> Vec<&NodeConfig> {
        self.nodes.values().filter(|n| n.node_type == NodeType::Orbital).collect()
    }

    /// Total compute capacity across all nodes.
    pub fn total_compute_tflops(&self) -> f64 {
        self.nodes.values().map(|n| n.compute_tflops).sum()
    }

    /// Total ground compute capacity.
    pub fn ground_compute_tflops(&self) -> f64 {
        self.ground_nodes().iter().map(|n| n.compute_tflops).sum()
    }

    /// Total orbital compute capacity.
    pub fn orbital_compute_tflops(&self) -> f64 {
        self.orbital_nodes().iter().map(|n| n.compute_tflops).sum()
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

/// Metrics for distributed training across Earth-space infrastructure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrainingMetrics {
    /// Total training steps completed
    pub total_steps: u64,
    /// Total samples processed
    pub total_samples: u64,
    /// Total epochs completed
    pub total_epochs: f64,
    /// Bytes uploaded
    pub bytes_uploaded: u64,
    /// Bytes downloaded
    pub bytes_downloaded: u64,
    /// Number of sync operations
    pub sync_count: u64,
    /// Compute time in seconds
    pub compute_time_s: f64,
    /// Communication time in seconds
    pub communication_time_s: f64,
    /// Idle time in seconds
    pub idle_time_s: f64,
    /// Loss history
    pub loss_history: Vec<f64>,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Sparsity achieved
    pub sparsity_achieved: f64,
}

impl TrainingMetrics {
    /// Create new training metrics.
    pub fn new() -> Self {
        Self {
            compression_ratio: 1.0,
            ..Default::default()
        }
    }

    /// Record end of a training step.
    pub fn end_step(&mut self, loss: Option<f64>, samples: u64, duration_s: f64) {
        self.compute_time_s += duration_s;
        self.total_steps += 1;
        self.total_samples += samples;
        if let Some(l) = loss {
            self.loss_history.push(l);
        }
    }

    /// Record a synchronization event.
    pub fn record_sync(&mut self, bytes_up: u64, bytes_down: u64, duration_s: f64) {
        self.bytes_uploaded += bytes_up;
        self.bytes_downloaded += bytes_down;
        self.communication_time_s += duration_s;
        self.sync_count += 1;
    }

    /// Total bytes transferred.
    pub fn total_bytes_transferred(&self) -> u64 {
        self.bytes_uploaded + self.bytes_downloaded
    }

    /// Compute efficiency (compute time / total time).
    pub fn compute_efficiency(&self) -> f64 {
        let total = self.compute_time_s + self.communication_time_s + self.idle_time_s;
        if total == 0.0 {
            0.0
        } else {
            self.compute_time_s / total
        }
    }

    /// Communication overhead (communication / compute).
    pub fn communication_overhead(&self) -> f64 {
        if self.compute_time_s == 0.0 {
            f64::INFINITY
        } else {
            self.communication_time_s / self.compute_time_s
        }
    }

    /// Average loss over all steps.
    pub fn average_loss(&self) -> Option<f64> {
        if self.loss_history.is_empty() {
            None
        } else {
            Some(self.loss_history.iter().sum::<f64>() / self.loss_history.len() as f64)
        }
    }

    /// Most recent loss value.
    pub fn latest_loss(&self) -> Option<f64> {
        self.loss_history.last().copied()
    }

    /// Get a summary of training metrics.
    pub fn summary(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        map.insert("total_steps".to_string(), self.total_steps as f64);
        map.insert("total_samples".to_string(), self.total_samples as f64);
        map.insert("compute_time_s".to_string(), (self.compute_time_s * 100.0).round() / 100.0);
        map.insert("communication_time_s".to_string(), (self.communication_time_s * 100.0).round() / 100.0);
        map.insert("compute_efficiency".to_string(), (self.compute_efficiency() * 10000.0).round() / 10000.0);
        map.insert("total_bytes_transferred".to_string(), self.total_bytes_transferred() as f64);
        map.insert("sync_count".to_string(), self.sync_count as f64);
        map.insert("compression_ratio".to_string(), (self.compression_ratio * 10000.0).round() / 10000.0);
        if let Some(loss) = self.latest_loss() {
            map.insert("latest_loss".to_string(), loss);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_config() {
        let orbital = NodeConfig::orbital("sat-1", 550.0, 10.0);
        assert_eq!(orbital.node_type, NodeType::Orbital);
        assert_eq!(orbital.orbit_altitude_km, Some(550.0));

        let ground = NodeConfig::ground("gs-1", 51.5, -0.1, 100.0);
        assert_eq!(ground.node_type, NodeType::Ground);
        assert_eq!(ground.location, Some((51.5, -0.1)));
    }

    #[test]
    fn test_topology() {
        let mut topo = Topology::new();
        topo.add_node(NodeConfig::orbital("sat-1", 550.0, 10.0));
        topo.add_node(NodeConfig::ground("gs-1", 51.5, -0.1, 100.0));

        assert_eq!(topo.node_count(), 2);
        assert_eq!(topo.orbital_nodes().len(), 1);
        assert_eq!(topo.ground_nodes().len(), 1);
        assert!((topo.total_compute_tflops() - 110.0).abs() < 0.1);
    }

    #[test]
    fn test_training_metrics() {
        let mut metrics = TrainingMetrics::new();
        metrics.end_step(Some(0.5), 32, 0.1);
        metrics.end_step(Some(0.4), 32, 0.1);
        metrics.record_sync(1000, 500, 0.05);

        assert_eq!(metrics.total_steps, 2);
        assert_eq!(metrics.total_samples, 64);
        assert_eq!(metrics.sync_count, 1);
        assert!((metrics.average_loss().unwrap() - 0.45).abs() < 0.01);
    }
}
