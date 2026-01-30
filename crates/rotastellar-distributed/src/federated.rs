//! Federated learning components for Earth-space distributed training.
//!
//! subhadipmitra@: This is the Rust port of our federated learning module. The algorithms
//! are identical to Python/TS to ensure cross-language consistency. We verified this using
//! shared test vectors (see /test-vectors/compression_tests.json).
//!
//! The compression here is aggressive by design - LEO uplinks are often 10-50Mbps with
//! 20-40ms latency, so we need to minimize data transfer.
//!
//! References:
//! - "Communication-Efficient Learning" (McMahan et al., 2017)
//! - "Deep Gradient Compression" (Lin et al., 2018)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO(subhadipmitra): Add SIMD optimization for the quantization loop
// TODO: Benchmark against tch-rs (PyTorch bindings) gradient compression

/// Gradient compression method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionMethod {
    /// No compression
    None,
    /// Top-K sparsification only
    TopK,
    /// Top-K with quantization
    TopKQuantized,
    /// Random-K sparsification
    RandomK,
    /// Quantization only
    Quantization,
}

/// Strategy for aggregating gradients from multiple nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationStrategy {
    /// Standard FedAvg
    FedAvg,
    /// Asynchronous FedAvg
    AsyncFedAvg,
    /// Weighted average based on sample counts
    WeightedAvg,
}

/// Configuration for gradient compression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression method to use
    pub method: CompressionMethod,
    /// For Top-K methods, fraction of gradients to keep (e.g., 0.01 = top 1%)
    pub k_ratio: f64,
    /// Number of bits for quantization (8, 4, or 2)
    pub quantization_bits: u8,
    /// Whether to accumulate compression error for future rounds
    pub error_feedback: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self::balanced()
    }
}

impl CompressionConfig {
    /// Create a new compression configuration.
    pub fn new(method: CompressionMethod, k_ratio: f64, quantization_bits: u8) -> Self {
        Self {
            method,
            k_ratio,
            quantization_bits,
            error_feedback: true,
        }
    }

    /// High compression configuration (1000x+).
    pub fn high_compression() -> Self {
        Self {
            method: CompressionMethod::TopKQuantized,
            k_ratio: 0.001,
            quantization_bits: 4,
            error_feedback: true,
        }
    }

    /// Balanced compression and accuracy.
    pub fn balanced() -> Self {
        Self {
            method: CompressionMethod::TopKQuantized,
            k_ratio: 0.01,
            quantization_bits: 8,
            error_feedback: true,
        }
    }

    /// Minimal compression for high-bandwidth links.
    pub fn low_compression() -> Self {
        Self {
            method: CompressionMethod::Quantization,
            k_ratio: 1.0,
            quantization_bits: 16,
            error_feedback: false,
        }
    }

    /// Theoretical compression ratio (smaller = more compression).
    pub fn theoretical_compression_ratio(&self) -> f64 {
        match self.method {
            CompressionMethod::None => 1.0,
            CompressionMethod::TopK => self.k_ratio * (1.0 + 32.0 / 32.0),
            CompressionMethod::TopKQuantized => {
                self.k_ratio * (self.quantization_bits as f64 / 32.0 + 32.0 / 32.0)
            }
            CompressionMethod::Quantization => self.quantization_bits as f64 / 32.0,
            CompressionMethod::RandomK => self.k_ratio,
        }
    }
}

/// Compressed gradient representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedGradient {
    /// Indices of non-zero values
    pub indices: Vec<usize>,
    /// Compressed values
    pub values: Vec<f64>,
    /// Original shape
    pub shape: Vec<usize>,
    /// Original size (number of elements)
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Achieved compression ratio
    pub compression_ratio: f64,
    /// Quantization bits used (if any)
    pub quantization_bits: Option<u8>,
}

impl CompressedGradient {
    /// Sparsity of the compressed gradient.
    pub fn sparsity(&self) -> f64 {
        1.0 - self.indices.len() as f64 / self.original_size as f64
    }
}

/// Compress gradients for bandwidth-efficient synchronization.
pub struct GradientCompressor {
    config: CompressionConfig,
    error_accumulator: Option<Vec<f64>>,
}

impl GradientCompressor {
    /// Create a new gradient compressor.
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            error_accumulator: None,
        }
    }

    /// Compress gradients.
    pub fn compress(&mut self, gradients: &[f64]) -> CompressedGradient {
        let original_size = gradients.len();
        let mut working: Vec<f64> = gradients.to_vec();

        // Apply error feedback if enabled
        if self.config.error_feedback {
            if let Some(ref errors) = self.error_accumulator {
                for (w, e) in working.iter_mut().zip(errors.iter()) {
                    *w += e;
                }
            }
        }

        if self.config.method == CompressionMethod::None {
            return CompressedGradient {
                indices: (0..original_size).collect(),
                values: working,
                shape: vec![original_size],
                original_size,
                compressed_size: original_size * 4,
                compression_ratio: 1.0,
                quantization_bits: None,
            };
        }

        let k = (original_size as f64 * self.config.k_ratio).ceil() as usize;
        let k = k.max(1).min(original_size);

        let (indices, mut values) = match self.config.method {
            CompressionMethod::TopK | CompressionMethod::TopKQuantized => {
                // Top-K selection
                let mut indexed: Vec<(usize, f64, f64)> = working
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| (i, v.abs(), v))
                    .collect();
                indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                let selected: Vec<_> = indexed.into_iter().take(k).collect();
                let indices: Vec<usize> = selected.iter().map(|x| x.0).collect();
                let values: Vec<f64> = selected.iter().map(|x| x.2).collect();
                (indices, values)
            }
            _ => {
                // Random-K or other methods
                use std::collections::HashSet;
                let mut selected = HashSet::new();
                let mut rng_idx = 0;
                while selected.len() < k {
                    let idx = (rng_idx * 1103515245 + 12345) % original_size;
                    selected.insert(idx);
                    rng_idx += 1;
                }
                let indices: Vec<usize> = selected.into_iter().collect();
                let values: Vec<f64> = indices.iter().map(|&i| working[i]).collect();
                (indices, values)
            }
        };

        // Apply quantization if needed
        if self.config.method == CompressionMethod::TopKQuantized && !values.is_empty() {
            values = self.quantize(&values);
        }

        // Calculate compression error for error feedback
        if self.config.error_feedback {
            let mut reconstructed = vec![0.0; original_size];
            for (i, &idx) in indices.iter().enumerate() {
                reconstructed[idx] = values[i];
            }
            self.error_accumulator = Some(
                working
                    .iter()
                    .zip(reconstructed.iter())
                    .map(|(w, r)| w - r)
                    .collect(),
            );
        }

        // Calculate compressed size
        let bits_per_value = if self.config.method == CompressionMethod::TopKQuantized {
            self.config.quantization_bits as usize
        } else {
            32
        };
        let compressed_bits = indices.len() * (32 + bits_per_value);
        let compressed_size = compressed_bits / 8;

        CompressedGradient {
            indices,
            values,
            shape: vec![original_size],
            original_size,
            compressed_size,
            compression_ratio: compressed_size as f64 / (original_size * 4) as f64,
            quantization_bits: if self.config.method == CompressionMethod::TopKQuantized {
                Some(self.config.quantization_bits)
            } else {
                None
            },
        }
    }

    /// Decompress gradients back to dense representation.
    pub fn decompress(&self, compressed: &CompressedGradient) -> Vec<f64> {
        let mut result = vec![0.0; compressed.original_size];
        for (i, &idx) in compressed.indices.iter().enumerate() {
            result[idx] = compressed.values[i];
        }
        result
    }

    fn quantize(&self, values: &[f64]) -> Vec<f64> {
        // subhadipmitra@: Linear quantization - matches Python/TS implementations exactly.
        // Could use SIMD here but keeping it simple for now.
        if values.is_empty() {
            return values.to_vec();
        }

        let min_val = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range_val = max_val - min_val;

        if range_val == 0.0 {
            return values.to_vec();
        }

        // NOTE(subhadipmitra): Using 1 << bits is safe here since quantization_bits is u8 (max 255)
        // and we only support 2, 4, 8, 16, 32 bit quantization anyway.
        let levels = (1u64 << self.config.quantization_bits) as f64;
        let scale = range_val / (levels - 1.0);

        // TODO(subhadipmitra): Consider stochastic rounding for better convergence
        values
            .iter()
            .map(|&v| {
                let q = ((v - min_val) / scale).round();
                min_val + q * scale
            })
            .collect()
    }
}

/// Client for federated learning on Earth or orbital nodes.
pub struct FederatedClient {
    /// Node ID
    pub node_id: String,
    /// Node type
    pub node_type: String,
    /// Compression config
    pub compression: CompressionConfig,
    compressor: GradientCompressor,
    local_steps: u64,
}

impl FederatedClient {
    /// Create a new federated client.
    pub fn new(node_id: &str, compression: Option<CompressionConfig>, node_type: &str) -> Self {
        let config = compression.unwrap_or_default();
        Self {
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
            compressor: GradientCompressor::new(config.clone()),
            compression: config,
            local_steps: 0,
        }
    }

    /// Create an orbital client with default compression.
    pub fn orbital(node_id: &str) -> Self {
        Self::new(node_id, Some(CompressionConfig::balanced()), "orbital")
    }

    /// Create a ground client with default compression.
    pub fn ground(node_id: &str) -> Self {
        Self::new(node_id, Some(CompressionConfig::low_compression()), "ground")
    }

    /// Compute gradients (simulation).
    pub fn compute_gradients(&mut self, model_params: &[f64], _local_data: &[f64]) -> Vec<f64> {
        self.local_steps += 1;
        // Simulated gradient computation
        model_params
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let seed = (self.local_steps as f64 * 1000.0 + i as f64).sin();
                seed * 0.1
            })
            .collect()
    }

    /// Compress gradients for transmission.
    pub fn compress(&mut self, gradients: &[f64]) -> CompressedGradient {
        self.compressor.compress(gradients)
    }

    /// Apply aggregated update to local model.
    pub fn apply_update(&self, model_params: &[f64], update: &[f64], lr: f64) -> Vec<f64> {
        model_params
            .iter()
            .zip(update.iter())
            .map(|(p, u)| p - u * lr)
            .collect()
    }

    /// Get client statistics.
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("node_id".to_string(), self.node_id.clone());
        stats.insert("node_type".to_string(), self.node_type.clone());
        stats.insert("local_steps".to_string(), self.local_steps.to_string());
        stats.insert(
            "compression_ratio".to_string(),
            format!("{:.4}", self.compression.theoretical_compression_ratio()),
        );
        stats
    }
}

/// Central aggregator for gradient synchronization.
pub struct GradientAggregator {
    /// Aggregation strategy
    pub strategy: AggregationStrategy,
    /// Minimum participants required
    pub min_participants: usize,
    /// Model size (if known)
    pub model_size: Option<usize>,
    pending_gradients: HashMap<String, (CompressedGradient, u64)>,
    round: u64,
}

impl GradientAggregator {
    /// Create a new gradient aggregator.
    pub fn new(strategy: AggregationStrategy, min_participants: usize) -> Self {
        Self {
            strategy,
            min_participants,
            model_size: None,
            pending_gradients: HashMap::new(),
            round: 0,
        }
    }

    /// Receive gradients from a node.
    pub fn receive_gradients(&mut self, node_id: &str, gradients: CompressedGradient, samples: u64) {
        self.pending_gradients
            .insert(node_id.to_string(), (gradients, samples));
    }

    /// Number of nodes that have submitted gradients.
    pub fn num_participants(&self) -> usize {
        self.pending_gradients.len()
    }

    /// Check if enough participants for aggregation.
    pub fn ready_to_aggregate(&self) -> bool {
        self.num_participants() >= self.min_participants
    }

    /// Aggregate gradients using configured strategy.
    pub fn aggregate(&mut self) -> Result<Vec<f64>, &'static str> {
        if self.pending_gradients.is_empty() {
            return Err("No gradients to aggregate");
        }

        let model_size = self.model_size.unwrap_or_else(|| {
            self.pending_gradients
                .values()
                .next()
                .map(|(g, _)| g.original_size)
                .unwrap_or(0)
        });

        let result = match self.strategy {
            AggregationStrategy::FedAvg | AggregationStrategy::WeightedAvg => {
                self.fed_avg(model_size)
            }
            AggregationStrategy::AsyncFedAvg => self.async_fed_avg(model_size),
        };

        self.pending_gradients.clear();
        self.round += 1;
        Ok(result)
    }

    fn fed_avg(&self, model_size: usize) -> Vec<f64> {
        let total_samples: u64 = self.pending_gradients.values().map(|(_, s)| s).sum();
        let mut aggregated = vec![0.0; model_size];

        for (grad, samples) in self.pending_gradients.values() {
            let weight = *samples as f64 / total_samples as f64;
            for (i, &idx) in grad.indices.iter().enumerate() {
                aggregated[idx] += grad.values[i] * weight;
            }
        }

        aggregated
    }

    fn async_fed_avg(&self, model_size: usize) -> Vec<f64> {
        let n = self.pending_gradients.len() as f64;
        let mut aggregated = vec![0.0; model_size];

        for (grad, _) in self.pending_gradients.values() {
            for (i, &idx) in grad.indices.iter().enumerate() {
                aggregated[idx] += grad.values[i] / n;
            }
        }

        aggregated
    }

    /// Get aggregator statistics.
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("strategy".to_string(), format!("{:?}", self.strategy));
        stats.insert("round".to_string(), self.round.to_string());
        stats.insert(
            "pending_participants".to_string(),
            self.num_participants().to_string(),
        );
        stats.insert(
            "min_participants".to_string(),
            self.min_participants.to_string(),
        );
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config() {
        let balanced = CompressionConfig::balanced();
        assert_eq!(balanced.method, CompressionMethod::TopKQuantized);
        assert!((balanced.k_ratio - 0.01).abs() < 0.001);

        let high = CompressionConfig::high_compression();
        assert!(high.theoretical_compression_ratio() < balanced.theoretical_compression_ratio());
    }

    #[test]
    fn test_gradient_compressor() {
        let config = CompressionConfig::balanced();
        let mut compressor = GradientCompressor::new(config);

        let gradients: Vec<f64> = (0..1000).map(|i| (i as f64).sin() * 0.1).collect();
        let compressed = compressor.compress(&gradients);

        assert!(compressed.indices.len() <= 100); // ~1% of 1000
        assert!(compressed.compression_ratio < 0.5);

        let decompressed = compressor.decompress(&compressed);
        assert_eq!(decompressed.len(), gradients.len());
    }

    #[test]
    fn test_federated_client() {
        let mut client = FederatedClient::orbital("sat-1");
        let model_params: Vec<f64> = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let gradients = client.compute_gradients(&model_params, &[]);

        assert_eq!(gradients.len(), model_params.len());

        let compressed = client.compress(&gradients);
        assert!(compressed.original_size == gradients.len());
    }

    #[test]
    fn test_gradient_aggregator() {
        let mut aggregator = GradientAggregator::new(AggregationStrategy::FedAvg, 2);

        let grad1 = CompressedGradient {
            indices: vec![0, 1, 2],
            values: vec![0.1, 0.2, 0.3],
            shape: vec![10],
            original_size: 10,
            compressed_size: 24,
            compression_ratio: 0.6,
            quantization_bits: None,
        };

        let grad2 = CompressedGradient {
            indices: vec![1, 2, 3],
            values: vec![0.2, 0.4, 0.1],
            shape: vec![10],
            original_size: 10,
            compressed_size: 24,
            compression_ratio: 0.6,
            quantization_bits: None,
        };

        aggregator.receive_gradients("node-1", grad1, 100);
        aggregator.receive_gradients("node-2", grad2, 100);

        assert!(aggregator.ready_to_aggregate());

        let result = aggregator.aggregate().unwrap();
        assert_eq!(result.len(), 10);
    }
}
