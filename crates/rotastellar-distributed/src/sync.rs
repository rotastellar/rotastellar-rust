//! Sync scheduling across ground station passes.
//!
//! subhadipmitra@: Ground station contact windows are the bottleneck for Earth-space
//! data transfer. A typical LEO satellite gets ~10-15 minutes of contact per pass,
//! with 4-6 passes per day over a given ground station.
//!
//! This module implements priority-based scheduling to ensure critical data (e.g.,
//! gradient updates for time-sensitive training) gets transmitted first.
//!
//! Real implementations would integrate with:
//! - Space-Track for TLE data
//! - GMAT or STK for precise pass predictions
//! - AWS Ground Station or Azure Orbital for actual antenna scheduling

use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

// TODO(subhadipmitra): Add actual pass prediction using SGP4
// TODO: Integrate with ground station APIs (AWS/Azure/KSAT)
// NOTE: Orbital period calculation assumes circular orbit (good enough for LEO)

/// Priority level for sync operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// Ground station configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundStation {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f64,
    pub bandwidth_mbps: f64,
    pub min_elevation_deg: f64,
}

impl GroundStation {
    /// Create a new ground station.
    pub fn new(name: &str, lat: f64, lon: f64) -> Self {
        Self {
            name: name.to_string(),
            latitude: lat,
            longitude: lon,
            elevation_m: 0.0,
            bandwidth_mbps: 100.0,
            min_elevation_deg: 5.0,
        }
    }

    /// Svalbard station.
    pub fn svalbard() -> Self {
        Self::new("Svalbard", 78.2306, 15.3894)
    }

    /// Kourou station.
    pub fn kourou() -> Self {
        Self::new("Kourou", 5.2378, -52.7683)
    }

    /// Default network of stations.
    pub fn default_network() -> Vec<Self> {
        vec![Self::svalbard(), Self::kourou()]
    }
}

/// A sync task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTask {
    pub task_id: String,
    pub node_id: String,
    pub data_size_bytes: u64,
    pub priority: Priority,
    pub description: String,
}

impl PartialEq for SyncTask {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl Eq for SyncTask {}

impl PartialOrd for SyncTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SyncTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower priority value = higher priority
        (other.priority as u8).cmp(&(self.priority as u8))
    }
}

/// Priority queue for sync operations.
#[derive(Debug, Default)]
pub struct PriorityQueue {
    heap: BinaryHeap<SyncTask>,
    counter: u64,
}

impl PriorityQueue {
    /// Create a new priority queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task to the queue.
    pub fn add_task(&mut self, node_id: &str, data_size_bytes: u64, priority: Priority, description: &str) -> String {
        self.counter += 1;
        let task_id = format!("task_{}", self.counter);
        let task = SyncTask {
            task_id: task_id.clone(),
            node_id: node_id.to_string(),
            data_size_bytes,
            priority,
            description: description.to_string(),
        };
        self.heap.push(task);
        task_id
    }

    /// Pop the highest priority task.
    pub fn pop_task(&mut self) -> Option<SyncTask> {
        self.heap.pop()
    }

    /// Peek at the highest priority task.
    pub fn peek_task(&self) -> Option<&SyncTask> {
        self.heap.peek()
    }

    /// Check if queue is empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Number of tasks in queue.
    pub fn size(&self) -> usize {
        self.heap.len()
    }

    /// Total bytes pending.
    pub fn total_bytes_pending(&self) -> u64 {
        self.heap.iter().map(|t| t.data_size_bytes).sum()
    }
}

/// Sync scheduler.
pub struct SyncScheduler {
    pub ground_stations: Vec<GroundStation>,
    pub orbit_altitude_km: f64,
    pub orbit_inclination_deg: f64,
    pub queue: PriorityQueue,
}

impl Default for SyncScheduler {
    fn default() -> Self {
        Self {
            ground_stations: GroundStation::default_network(),
            orbit_altitude_km: 550.0,
            orbit_inclination_deg: 51.6,
            queue: PriorityQueue::new(),
        }
    }
}

impl SyncScheduler {
    /// Create a new sync scheduler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Orbital period in minutes.
    pub fn orbital_period_minutes(&self) -> f64 {
        let earth_radius = 6371.0;
        let earth_mu = 398600.4418;
        let a = earth_radius + self.orbit_altitude_km;
        let period_s = 2.0 * std::f64::consts::PI * (a.powi(3) / earth_mu).sqrt();
        period_s / 60.0
    }

    /// Orbits per day.
    pub fn orbits_per_day(&self) -> f64 {
        (24.0 * 60.0) / self.orbital_period_minutes()
    }

    /// Schedule a sync operation.
    pub fn schedule_sync(&mut self, node_id: &str, data_size_bytes: u64, priority: Priority, description: &str) -> String {
        self.queue.add_task(node_id, data_size_bytes, priority, description)
    }

    /// Get schedule summary.
    pub fn get_schedule_summary(&self) -> std::collections::HashMap<String, f64> {
        let mut summary = std::collections::HashMap::new();
        summary.insert("pending_tasks".to_string(), self.queue.size() as f64);
        summary.insert("pending_data_mb".to_string(), self.queue.total_bytes_pending() as f64 / (1024.0 * 1024.0));
        summary.insert("orbital_period_min".to_string(), self.orbital_period_minutes());
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_queue() {
        let mut queue = PriorityQueue::new();
        queue.add_task("node-1", 1000, Priority::Low, "low task");
        queue.add_task("node-2", 2000, Priority::Critical, "critical task");
        queue.add_task("node-3", 1500, Priority::High, "high task");

        let task = queue.pop_task().unwrap();
        assert_eq!(task.priority, Priority::Critical);
    }

    #[test]
    fn test_sync_scheduler() {
        let mut scheduler = SyncScheduler::new();
        scheduler.schedule_sync("orbital-1", 1024 * 1024, Priority::High, "Upload gradients");

        assert_eq!(scheduler.queue.size(), 1);
        assert!(scheduler.orbital_period_minutes() > 90.0);
    }
}
