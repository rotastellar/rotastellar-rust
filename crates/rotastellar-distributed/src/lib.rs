//! # RotaStellar Distributed
//!
//! Distributed compute coordination for Earth-space AI workloads.
//!
//! ## Overview
//!
//! This crate provides tools for coordinating AI workloads across Earth and
//! orbital infrastructure, including:
//!
//! - **Federated Learning** — Train models across distributed nodes with gradient compression
//! - **Model Partitioning** — Optimal layer placement across ground and orbital compute
//! - **Sync Scheduling** — Ground station pass planning and priority queuing
//! - **Space Mesh** — ISL routing for orbital node communication
//!
//! ## Example
//!
//! ```rust
//! use rotastellar_distributed::{
//!     FederatedClient, CompressionConfig, GradientAggregator, AggregationStrategy,
//!     ModelProfile, PartitionOptimizer, OptimizationObjective,
//!     SyncScheduler, Priority,
//!     SpaceMesh, create_constellation,
//! };
//!
//! // Federated learning with gradient compression
//! let mut client = FederatedClient::orbital("orbital-1");
//! let model_params = vec![0.1, 0.2, 0.3, 0.4, 0.5];
//! let gradients = client.compute_gradients(&model_params, &[]);
//! let compressed = client.compress(&gradients);
//! println!("Compression ratio: {:.4}", compressed.compression_ratio);
//!
//! // Model partitioning
//! let model = ModelProfile::create_transformer(6, 768, 50000, 512);
//! let optimizer = PartitionOptimizer::default();
//! let plan = optimizer.optimize(&model, OptimizationObjective::Balance);
//! println!("Ground layers: {}, Orbital layers: {}",
//!     plan.ground_layers().len(), plan.orbital_layers().len());
//!
//! // Sync scheduling
//! let mut scheduler = SyncScheduler::new();
//! scheduler.schedule_sync("orbital-1", 1024*1024, Priority::High, "Upload gradients");
//!
//! // Space mesh routing
//! let mesh = create_constellation("test", 4, 10, 550.0, 53.0, 5000.0);
//! let route = mesh.find_route("test_P0_S0", "test_P2_S5");
//! if route.is_valid() {
//!     println!("Route: {:?}, Latency: {:.1}ms", route.path, route.total_latency_ms);
//! }
//! ```

#![doc(html_root_url = "https://docs.rs/rotastellar-distributed/0.1.0")]
#![warn(missing_docs)]

pub mod core;
pub mod federated;
pub mod mesh;
pub mod partitioning;
pub mod sync;

// Re-export commonly used items
pub use core::{NodeConfig, NodeType, Topology, TrainingMetrics};

pub use federated::{
    AggregationStrategy, CompressedGradient, CompressionConfig, CompressionMethod,
    FederatedClient, GradientAggregator, GradientCompressor,
};

pub use partitioning::{
    LayerPlacement, LayerProfile, LayerType, ModelProfile, OptimizationObjective,
    PartitionOptimizer, PartitionPlan, PlacementLocation,
};

pub use sync::{GroundStation, Priority, PriorityQueue, SyncScheduler, SyncTask};

pub use mesh::{create_constellation, ISLLink, LinkType, OrbitalNode, Route, SpaceMesh};

/// Current version of the crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_federated_integration() {
        let mut client = FederatedClient::orbital("sat-1");
        let params = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let grads = client.compute_gradients(&params, &[]);
        let compressed = client.compress(&grads);

        assert!(compressed.compression_ratio < 1.0);
    }

    #[test]
    fn test_partitioning_integration() {
        let model = ModelProfile::create_transformer(6, 768, 50000, 512);
        let optimizer = PartitionOptimizer::default();
        let plan = optimizer.optimize(&model, OptimizationObjective::Balance);

        assert!(!plan.placements.is_empty());
    }

    #[test]
    fn test_sync_integration() {
        let mut scheduler = SyncScheduler::new();
        let task_id = scheduler.schedule_sync("orbital-1", 1024 * 1024, Priority::High, "test");

        assert!(!task_id.is_empty());
        assert_eq!(scheduler.queue.size(), 1);
    }

    #[test]
    fn test_mesh_integration() {
        let mesh = create_constellation("test", 2, 4, 550.0, 53.0, 5000.0);
        let stats = mesh.get_mesh_stats();

        assert_eq!(stats.get("total_nodes"), Some(&8.0));
    }
}
