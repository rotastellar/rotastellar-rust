//! Model partitioning across Earth and orbital compute nodes.
//!
//! subhadipmitra@: The key insight here is that different layers have different
//! compute-to-communication ratios. Attention layers are compute-heavy (good for
//! bandwidth-limited orbital nodes), while embedding layers transfer a lot of data
//! (better to keep on ground).
//!
//! The optimizer finds the split point that minimizes total latency, accounting for:
//! - Compute time on ground vs. orbital nodes (10:1 ratio typically)
//! - Data transfer time for the "cut" between segments
//! - Propagation delay (~2ms for 550km altitude)
//!
//! This is related to pipeline parallelism in traditional distributed training,
//! but with much higher communication latency.

use serde::{Deserialize, Serialize};

// TODO(subhadipmitra): Add support for multiple split points (not just one)
// TODO: Consider memory constraints on orbital nodes (typically <16GB)
// FIXME: The latency model doesn't account for queueing delays

/// Type of neural network layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerType {
    Linear,
    Conv2d,
    Attention,
    Embedding,
    Normalization,
    Activation,
    Pooling,
    Other,
}

/// Where to place a layer for computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementLocation {
    Ground,
    Orbital,
    Split,
}

/// Objective for partition optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationObjective {
    MinimizeLatency,
    MinimizeBandwidth,
    Balance,
    MaximizeThroughput,
}

/// Profile of a single layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerProfile {
    pub name: String,
    pub layer_type: LayerType,
    pub params: u64,
    pub flops: u64,
    pub input_size: u64,
    pub output_size: u64,
}

/// Profile of a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub layers: Vec<LayerProfile>,
    pub name: String,
}

impl ModelProfile {
    /// Create a new model profile.
    pub fn new(name: &str) -> Self {
        Self {
            layers: Vec::new(),
            name: name.to_string(),
        }
    }

    /// Add a layer to the model.
    pub fn add_layer(&mut self, layer: LayerProfile) {
        self.layers.push(layer);
    }

    /// Create a typical transformer model profile.
    pub fn create_transformer(num_layers: usize, hidden_size: usize, vocab_size: usize, seq_length: usize) -> Self {
        let mut profile = Self::new("transformer");

        // Embedding
        profile.add_layer(LayerProfile {
            name: "embedding".to_string(),
            layer_type: LayerType::Embedding,
            params: (vocab_size * hidden_size) as u64,
            flops: (seq_length * hidden_size) as u64,
            input_size: (seq_length * 4) as u64,
            output_size: (seq_length * hidden_size * 4) as u64,
        });

        // Transformer layers
        for i in 0..num_layers {
            let attn_params = (4 * hidden_size * hidden_size) as u64;
            let attn_flops = (2 * seq_length * seq_length * hidden_size + 4 * seq_length * hidden_size * hidden_size) as u64;
            profile.add_layer(LayerProfile {
                name: format!("layer_{}_attention", i),
                layer_type: LayerType::Attention,
                params: attn_params,
                flops: attn_flops,
                input_size: (seq_length * hidden_size * 4) as u64,
                output_size: (seq_length * hidden_size * 4) as u64,
            });

            let ffn_params = (2 * hidden_size * 4 * hidden_size) as u64;
            let ffn_flops = (2 * seq_length * hidden_size * 4 * hidden_size) as u64;
            profile.add_layer(LayerProfile {
                name: format!("layer_{}_ffn", i),
                layer_type: LayerType::Linear,
                params: ffn_params,
                flops: ffn_flops,
                input_size: (seq_length * hidden_size * 4) as u64,
                output_size: (seq_length * hidden_size * 4) as u64,
            });
        }

        // Output
        profile.add_layer(LayerProfile {
            name: "output".to_string(),
            layer_type: LayerType::Linear,
            params: (hidden_size * vocab_size) as u64,
            flops: (seq_length * hidden_size * vocab_size) as u64,
            input_size: (seq_length * hidden_size * 4) as u64,
            output_size: (seq_length * vocab_size * 4) as u64,
        });

        profile
    }

    /// Total parameters.
    pub fn total_params(&self) -> u64 {
        self.layers.iter().map(|l| l.params).sum()
    }

    /// Total FLOPs.
    pub fn total_flops(&self) -> u64 {
        self.layers.iter().map(|l| l.flops).sum()
    }

    /// Number of layers.
    pub fn num_layers(&self) -> usize {
        self.layers.len()
    }
}

/// Placement decision for a layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerPlacement {
    pub layer_name: String,
    pub location: PlacementLocation,
    pub node_id: Option<String>,
    pub estimated_latency_ms: f64,
    pub data_transfer_bytes: u64,
}

/// Complete partitioning plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionPlan {
    pub model_name: String,
    pub placements: Vec<LayerPlacement>,
    pub total_latency_ms: f64,
    pub ground_orbital_transfers: u32,
    pub total_transfer_bytes: u64,
    pub objective: OptimizationObjective,
}

impl PartitionPlan {
    /// Get ground layers.
    pub fn ground_layers(&self) -> Vec<&LayerPlacement> {
        self.placements.iter().filter(|p| p.location == PlacementLocation::Ground).collect()
    }

    /// Get orbital layers.
    pub fn orbital_layers(&self) -> Vec<&LayerPlacement> {
        self.placements.iter().filter(|p| p.location == PlacementLocation::Orbital).collect()
    }
}

/// Optimize model partitioning.
pub struct PartitionOptimizer {
    pub ground_compute_tflops: f64,
    pub orbital_compute_tflops: f64,
    pub orbit_altitude_km: f64,
    pub uplink_bandwidth_mbps: f64,
    pub downlink_bandwidth_mbps: f64,
}

impl Default for PartitionOptimizer {
    fn default() -> Self {
        Self {
            ground_compute_tflops: 100.0,
            orbital_compute_tflops: 10.0,
            orbit_altitude_km: 550.0,
            uplink_bandwidth_mbps: 100.0,
            downlink_bandwidth_mbps: 200.0,
        }
    }
}

impl PartitionOptimizer {
    /// Create a new partition optimizer.
    pub fn new(ground_tflops: f64, orbital_tflops: f64) -> Self {
        Self {
            ground_compute_tflops: ground_tflops,
            orbital_compute_tflops: orbital_tflops,
            ..Default::default()
        }
    }

    /// Optimize partition for the model.
    pub fn optimize(&self, model: &ModelProfile, objective: OptimizationObjective) -> PartitionPlan {
        let split_idx = match objective {
            OptimizationObjective::MinimizeLatency => self.find_best_latency_split(model),
            OptimizationObjective::MinimizeBandwidth => self.find_min_bandwidth_split(model),
            _ => self.find_balanced_split(model),
        };

        self.create_plan(model, split_idx, objective)
    }

    fn find_best_latency_split(&self, model: &ModelProfile) -> usize {
        let mut best_idx = 0;
        let mut best_latency = f64::INFINITY;

        for i in 0..=model.layers.len() {
            let plan = self.create_plan(model, i, OptimizationObjective::MinimizeLatency);
            if plan.total_latency_ms < best_latency {
                best_latency = plan.total_latency_ms;
                best_idx = i;
            }
        }
        best_idx
    }

    fn find_min_bandwidth_split(&self, model: &ModelProfile) -> usize {
        let mut min_idx = 0;
        let mut min_size = u64::MAX;

        for i in 0..model.layers.len() {
            let size = model.layers[i].output_size;
            if size < min_size {
                min_size = size;
                min_idx = i + 1;
            }
        }
        min_idx
    }

    fn find_balanced_split(&self, model: &ModelProfile) -> usize {
        let total_flops = model.total_flops() as f64;
        let target = total_flops * self.ground_compute_tflops / (self.ground_compute_tflops + self.orbital_compute_tflops);

        let mut cumulative: f64 = 0.0;
        for (i, layer) in model.layers.iter().enumerate() {
            cumulative += layer.flops as f64;
            if cumulative >= target {
                return i + 1;
            }
        }
        model.layers.len()
    }

    fn create_plan(&self, model: &ModelProfile, split_idx: usize, objective: OptimizationObjective) -> PartitionPlan {
        let mut placements = Vec::new();
        let mut total_latency_ms = 0.0;
        let mut total_transfer: u64 = 0;
        let mut num_transfers: u32 = 0;

        for (i, layer) in model.layers.iter().enumerate() {
            let location = if i < split_idx { PlacementLocation::Ground } else { PlacementLocation::Orbital };
            let compute_tflops = if location == PlacementLocation::Ground {
                self.ground_compute_tflops
            } else {
                self.orbital_compute_tflops
            };

            let mut layer_latency_ms = (layer.flops as f64 / (compute_tflops * 1e12)) * 1000.0;
            let mut transfer_bytes: u64 = 0;

            if i == split_idx && split_idx > 0 && split_idx < model.layers.len() {
                transfer_bytes = layer.input_size;
                let transfer_latency = (transfer_bytes as f64 * 8.0) / (self.uplink_bandwidth_mbps * 1e6) * 1000.0;
                let propagation = (self.orbit_altitude_km / 299792.458) * 1000.0;
                layer_latency_ms += transfer_latency + propagation;
                total_transfer += transfer_bytes;
                num_transfers += 1;
            }

            placements.push(LayerPlacement {
                layer_name: layer.name.clone(),
                location,
                node_id: None,
                estimated_latency_ms: layer_latency_ms,
                data_transfer_bytes: transfer_bytes,
            });

            total_latency_ms += layer_latency_ms;
        }

        PartitionPlan {
            model_name: model.name.clone(),
            placements,
            total_latency_ms,
            ground_orbital_transfers: num_transfers,
            total_transfer_bytes: total_transfer,
            objective,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_profile() {
        let model = ModelProfile::create_transformer(6, 768, 50000, 512);
        assert!(model.num_layers() > 10);
        assert!(model.total_params() > 0);
    }

    #[test]
    fn test_partition_optimizer() {
        let model = ModelProfile::create_transformer(6, 768, 50000, 512);
        let optimizer = PartitionOptimizer::default();
        let plan = optimizer.optimize(&model, OptimizationObjective::Balance);

        assert!(!plan.placements.is_empty());
        assert!(plan.total_latency_ms > 0.0);
    }
}
