//! ISL routing mesh for orbital node communication.
//!
//! subhadipmitra@: This module models inter-satellite link (ISL) networks similar to
//! Starlink's laser mesh. Each satellite can maintain multiple ISL connections based
//! on distance and line-of-sight constraints.
//!
//! The routing uses Dijkstra's algorithm which is O(E log V). For large constellations
//! (>1000 nodes), consider switching to A* with great-circle heuristic or pre-computed
//! routing tables updated on topology changes.
//!
//! Key assumptions:
//! - Optical ISL (not RF) so we use speed of light in vacuum
//! - Simplified orbital mechanics (circular orbits, no perturbations)
//! - Static topology snapshot (real system would update every few seconds)

use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

// TODO(subhadipmitra): Add support for ground station nodes in the mesh
// TODO: Implement A* routing with angular distance heuristic for large constellations
// FIXME: The LOS calculation is approximate - need proper ray-sphere intersection

/// Type of communication link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    Optical,
    Rf,
    Hybrid,
}

/// An orbital compute node in the mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitalNode {
    pub node_id: String,
    pub orbit_altitude_km: f64,
    pub orbit_inclination_deg: f64,
    pub raan_deg: f64,
    pub mean_anomaly_deg: f64,
    pub isl_range_km: f64,
    pub isl_bandwidth_gbps: f64,
    pub compute_tflops: f64,
}

impl OrbitalNode {
    /// Create a new orbital node.
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            orbit_altitude_km: 550.0,
            orbit_inclination_deg: 51.6,
            raan_deg: 0.0,
            mean_anomaly_deg: 0.0,
            isl_range_km: 5000.0,
            isl_bandwidth_gbps: 10.0,
            compute_tflops: 10.0,
        }
    }

    /// Set orbital parameters.
    pub fn with_orbit(mut self, raan_deg: f64, mean_anomaly_deg: f64) -> Self {
        self.raan_deg = raan_deg;
        self.mean_anomaly_deg = mean_anomaly_deg;
        self
    }
}

/// Inter-satellite link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISLLink {
    pub source_id: String,
    pub target_id: String,
    pub distance_km: f64,
    pub bandwidth_gbps: f64,
    pub latency_ms: f64,
    pub link_type: LinkType,
    pub active: bool,
}

/// A route through the mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub source_id: String,
    pub destination_id: String,
    pub path: Vec<String>,
    pub total_distance_km: f64,
    pub total_latency_ms: f64,
    pub min_bandwidth_gbps: f64,
    pub num_hops: usize,
}

impl Route {
    /// Check if route is valid.
    pub fn is_valid(&self) -> bool {
        self.path.len() >= 2
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DijkstraState {
    cost: f64,
    node_id: String,
}

impl Eq for DijkstraState {}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

/// ISL routing mesh.
pub struct SpaceMesh {
    pub default_isl_range_km: f64,
    nodes: HashMap<String, OrbitalNode>,
    links: HashMap<String, ISLLink>,
    adjacency: HashMap<String, HashSet<String>>,
}

impl Default for SpaceMesh {
    fn default() -> Self {
        Self::new(5000.0)
    }
}

impl SpaceMesh {
    const SPEED_OF_LIGHT_KM_S: f64 = 299792.458;
    const EARTH_RADIUS_KM: f64 = 6371.0;

    /// Create a new space mesh.
    pub fn new(default_isl_range_km: f64) -> Self {
        Self {
            default_isl_range_km,
            nodes: HashMap::new(),
            links: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Add a node to the mesh.
    pub fn add_node(&mut self, node: OrbitalNode) {
        self.adjacency.insert(node.node_id.clone(), HashSet::new());
        self.nodes.insert(node.node_id.clone(), node);
    }

    /// Update the mesh topology.
    pub fn update_topology(&mut self) {
        self.links.clear();
        for adj in self.adjacency.values_mut() {
            adj.clear();
        }

        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();

        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let id1 = &node_ids[i];
                let id2 = &node_ids[j];
                let node1 = &self.nodes[id1];
                let node2 = &self.nodes[id2];

                let distance = self.calculate_distance(node1, node2);
                let max_range = node1.isl_range_km.min(node2.isl_range_km);

                if distance <= max_range && self.has_line_of_sight(node1, node2) {
                    let bandwidth = node1.isl_bandwidth_gbps.min(node2.isl_bandwidth_gbps);
                    let latency = (distance / Self::SPEED_OF_LIGHT_KM_S) * 1000.0;

                    let link1 = ISLLink {
                        source_id: id1.clone(),
                        target_id: id2.clone(),
                        distance_km: distance,
                        bandwidth_gbps: bandwidth,
                        latency_ms: latency,
                        link_type: LinkType::Optical,
                        active: true,
                    };

                    let link2 = ISLLink {
                        source_id: id2.clone(),
                        target_id: id1.clone(),
                        distance_km: distance,
                        bandwidth_gbps: bandwidth,
                        latency_ms: latency,
                        link_type: LinkType::Optical,
                        active: true,
                    };

                    self.links.insert(format!("{}-{}", id1, id2), link1);
                    self.links.insert(format!("{}-{}", id2, id1), link2);

                    self.adjacency.get_mut(id1).unwrap().insert(id2.clone());
                    self.adjacency.get_mut(id2).unwrap().insert(id1.clone());
                }
            }
        }
    }

    /// Find optimal route between two nodes.
    pub fn find_route(&self, source_id: &str, destination_id: &str) -> Route {
        if !self.nodes.contains_key(source_id) || !self.nodes.contains_key(destination_id) {
            return Route {
                source_id: source_id.to_string(),
                destination_id: destination_id.to_string(),
                path: vec![],
                total_distance_km: 0.0,
                total_latency_ms: 0.0,
                min_bandwidth_gbps: 0.0,
                num_hops: 0,
            };
        }

        if source_id == destination_id {
            return Route {
                source_id: source_id.to_string(),
                destination_id: destination_id.to_string(),
                path: vec![source_id.to_string()],
                total_distance_km: 0.0,
                total_latency_ms: 0.0,
                min_bandwidth_gbps: f64::INFINITY,
                num_hops: 0,
            };
        }

        let mut distances: HashMap<String, f64> = self.nodes.keys().map(|k| (k.clone(), f64::INFINITY)).collect();
        let mut predecessors: HashMap<String, Option<String>> = self.nodes.keys().map(|k| (k.clone(), None)).collect();
        let mut visited = HashSet::new();

        distances.insert(source_id.to_string(), 0.0);
        let mut pq = BinaryHeap::new();
        pq.push(DijkstraState {
            cost: 0.0,
            node_id: source_id.to_string(),
        });

        while let Some(DijkstraState { cost, node_id }) = pq.pop() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            if node_id == destination_id {
                break;
            }

            if let Some(neighbors) = self.adjacency.get(&node_id) {
                for neighbor_id in neighbors {
                    if visited.contains(neighbor_id) {
                        continue;
                    }

                    let link_key = format!("{}-{}", node_id, neighbor_id);
                    if let Some(link) = self.links.get(&link_key) {
                        if !link.active {
                            continue;
                        }

                        let new_cost = cost + link.latency_ms;
                        if new_cost < *distances.get(neighbor_id).unwrap_or(&f64::INFINITY) {
                            distances.insert(neighbor_id.clone(), new_cost);
                            predecessors.insert(neighbor_id.clone(), Some(node_id.clone()));
                            pq.push(DijkstraState {
                                cost: new_cost,
                                node_id: neighbor_id.clone(),
                            });
                        }
                    }
                }
            }
        }

        if distances.get(destination_id).unwrap_or(&f64::INFINITY) == &f64::INFINITY {
            return Route {
                source_id: source_id.to_string(),
                destination_id: destination_id.to_string(),
                path: vec![],
                total_distance_km: 0.0,
                total_latency_ms: 0.0,
                min_bandwidth_gbps: 0.0,
                num_hops: 0,
            };
        }

        let mut path = Vec::new();
        let mut current: Option<String> = Some(destination_id.to_string());
        while let Some(ref id) = current {
            path.push(id.clone());
            current = predecessors.get(id).and_then(|p| p.clone());
        }
        path.reverse();

        let mut total_distance = 0.0;
        let mut total_latency = 0.0;
        let mut min_bandwidth = f64::INFINITY;

        for i in 0..(path.len() - 1) {
            let link_key = format!("{}-{}", path[i], path[i + 1]);
            if let Some(link) = self.links.get(&link_key) {
                total_distance += link.distance_km;
                total_latency += link.latency_ms;
                min_bandwidth = min_bandwidth.min(link.bandwidth_gbps);
            }
        }

        Route {
            source_id: source_id.to_string(),
            destination_id: destination_id.to_string(),
            path,
            total_distance_km: (total_distance * 100.0).round() / 100.0,
            total_latency_ms: (total_latency * 1000.0).round() / 1000.0,
            min_bandwidth_gbps: if min_bandwidth == f64::INFINITY { 0.0 } else { min_bandwidth },
            num_hops: 0, // Will be set below
        }
    }

    /// Get mesh statistics.
    pub fn get_mesh_stats(&self) -> HashMap<String, f64> {
        let mut unique_links = HashSet::new();
        for link in self.links.values() {
            if link.active {
                let sorted = if link.source_id < link.target_id {
                    format!("{}-{}", link.source_id, link.target_id)
                } else {
                    format!("{}-{}", link.target_id, link.source_id)
                };
                unique_links.insert(sorted);
            }
        }

        let num_links = unique_links.len();
        let avg_links = if self.nodes.is_empty() { 0.0 } else { (2.0 * num_links as f64) / self.nodes.len() as f64 };

        let mut stats = HashMap::new();
        stats.insert("total_nodes".to_string(), self.nodes.len() as f64);
        stats.insert("active_links".to_string(), num_links as f64);
        stats.insert("avg_links_per_node".to_string(), (avg_links * 100.0).round() / 100.0);
        stats
    }

    fn calculate_distance(&self, node1: &OrbitalNode, node2: &OrbitalNode) -> f64 {
        let r1 = Self::EARTH_RADIUS_KM + node1.orbit_altitude_km;
        let r2 = Self::EARTH_RADIUS_KM + node2.orbit_altitude_km;

        let theta1 = node1.mean_anomaly_deg.to_radians();
        let theta2 = node2.mean_anomaly_deg.to_radians();

        let inc1 = node1.orbit_inclination_deg.to_radians();
        let inc2 = node2.orbit_inclination_deg.to_radians();

        let raan1 = node1.raan_deg.to_radians();
        let raan2 = node2.raan_deg.to_radians();

        let x1 = r1 * (raan1.cos() * theta1.cos() - raan1.sin() * theta1.sin() * inc1.cos());
        let y1 = r1 * (raan1.sin() * theta1.cos() + raan1.cos() * theta1.sin() * inc1.cos());
        let z1 = r1 * theta1.sin() * inc1.sin();

        let x2 = r2 * (raan2.cos() * theta2.cos() - raan2.sin() * theta2.sin() * inc2.cos());
        let y2 = r2 * (raan2.sin() * theta2.cos() + raan2.cos() * theta2.sin() * inc2.cos());
        let z2 = r2 * theta2.sin() * inc2.sin();

        ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt()
    }

    fn has_line_of_sight(&self, node1: &OrbitalNode, node2: &OrbitalNode) -> bool {
        let min_altitude = node1.orbit_altitude_km.min(node2.orbit_altitude_km);
        let distance = self.calculate_distance(node1, node2);
        let max_los = 2.0 * ((Self::EARTH_RADIUS_KM + min_altitude).powi(2) - Self::EARTH_RADIUS_KM.powi(2)).sqrt();
        distance <= max_los
    }
}

/// Create a Walker constellation mesh.
///
/// subhadipmitra@: Walker constellations are parameterized as i:t/p/f where:
/// - i = inclination
/// - t = total satellites
/// - p = number of orbital planes
/// - f = phasing factor (we compute this automatically)
///
/// This function creates a Walker Delta pattern which is common for global coverage
/// (used by Iridium, Starlink, etc.)
pub fn create_constellation(name: &str, num_planes: usize, sats_per_plane: usize, altitude_km: f64, inclination_deg: f64, isl_range_km: f64) -> SpaceMesh {
    let mut mesh = SpaceMesh::new(isl_range_km);

    for plane in 0..num_planes {
        // RAAN spacing for even coverage
        let raan = (360.0 / num_planes as f64) * plane as f64;

        for sat in 0..sats_per_plane {
            let mut mean_anomaly = (360.0 / sats_per_plane as f64) * sat as f64;
            // subhadipmitra@: Phase offset between planes prevents "seams" in coverage
            mean_anomaly += (360.0 / (num_planes * sats_per_plane) as f64) * plane as f64;

            let node_id = format!("{}_P{}_S{}", name, plane, sat);
            let node = OrbitalNode {
                node_id: node_id.clone(),
                orbit_altitude_km: altitude_km,
                orbit_inclination_deg: inclination_deg,
                raan_deg: raan,
                mean_anomaly_deg: mean_anomaly,
                isl_range_km,
                isl_bandwidth_gbps: 10.0,
                compute_tflops: 10.0,
            };
            mesh.add_node(node);
        }
    }

    mesh.update_topology();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_mesh() {
        let mut mesh = SpaceMesh::new(5000.0);
        mesh.add_node(OrbitalNode::new("sat-1").with_orbit(0.0, 0.0));
        mesh.add_node(OrbitalNode::new("sat-2").with_orbit(0.0, 30.0));
        mesh.add_node(OrbitalNode::new("sat-3").with_orbit(0.0, 60.0));
        mesh.update_topology();

        let stats = mesh.get_mesh_stats();
        assert_eq!(stats.get("total_nodes"), Some(&3.0));
    }

    #[test]
    fn test_create_constellation() {
        let mesh = create_constellation("test", 2, 4, 550.0, 53.0, 5000.0);
        let stats = mesh.get_mesh_stats();
        assert_eq!(stats.get("total_nodes"), Some(&8.0));
    }
}
