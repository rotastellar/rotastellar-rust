# rotastellar-distributed

**Distributed Computing for Space Infrastructure**

Federated learning, model partitioning, gradient synchronization, and mesh networking for orbital compute clusters.

## Installation

```toml
[dependencies]
rotastellar-distributed = "0.1"
```

## Quick Start

### Federated Learning

```rust
use rotastellar_distributed::{
    FederatedClient,
    GradientAggregator,
    AggregationStrategy,
    CompressionConfig,
    CompressionType,
};

fn main() {
    // Configure gradient compression for limited bandwidth
    let compression = CompressionConfig::new(CompressionType::TopK)
        .with_sparsity(0.99)  // Keep top 1% of gradients
        .with_error_feedback(true);

    // Create federated client
    let client = FederatedClient::new("sat-001", compression);

    // Compress gradients before transmission
    let gradients = vec![0.1, 0.2, -0.5, 0.01, 0.8];
    let compressed = client.compress(&gradients);
    println!("Compression ratio: {:.1}x", compressed.compression_ratio);

    // Server-side aggregation
    let aggregator = GradientAggregator::new(AggregationStrategy::FedAvg);
    let client_gradients = vec![grad1, grad2, grad3];
    let weights = vec![0.4, 0.3, 0.3];
    let aggregated = aggregator.aggregate(&client_gradients, &weights);
}
```

### Model Partitioning

```rust
use rotastellar_distributed::{
    ModelProfile,
    PartitionOptimizer,
    NodeConfig,
    NodeType,
    LayerProfile,
};

fn main() {
    // Profile your model
    let profile = ModelProfile::new(vec![
        LayerProfile::new("embedding", 100.0, 1e9),
        LayerProfile::new("transformer_1", 200.0, 5e9),
        LayerProfile::new("transformer_2", 200.0, 5e9),
        LayerProfile::new("output", 50.0, 1e8),
    ]);

    // Define available nodes
    let nodes = vec![
        NodeConfig::new("sat-001", NodeType::Satellite, 8.0, 2.0),
        NodeConfig::new("sat-002", NodeType::Satellite, 8.0, 2.0),
        NodeConfig::new("ground-001", NodeType::Ground, 32.0, 10.0),
    ];

    // Optimize partitioning
    let optimizer = PartitionOptimizer::new();
    let plan = optimizer.optimize(&profile, &nodes);
    println!("Partition plan: {:?}", plan.assignments);
    println!("Estimated latency: {:.1} ms", plan.estimated_latency_ms);
}
```

### Synchronization Scheduling

```rust
use rotastellar_distributed::{SyncScheduler, GroundStation};
use rotastellar::Position;

fn main() {
    // Define ground stations
    let stations = vec![
        GroundStation::new(
            "KSC",
            Position::new(28.5729, -80.6490, 0.0),
            100.0,  // uplink Mbps
            200.0,  // downlink Mbps
        ),
        GroundStation::new(
            "Svalbard",
            Position::new(78.2297, 15.3975, 0.0),
            150.0,
            300.0,
        ),
    ];

    // Create scheduler
    let scheduler = SyncScheduler::new(stations);

    // Get optimal sync windows
    let windows = scheduler.get_sync_windows("sat-001", 24);  // 24 hours
    for window in windows {
        println!("Station: {}", window.station.name);
        println!("Start: {}, Duration: {}s", window.start_time, window.duration_seconds);
        println!("Data capacity: {:.1} MB", window.data_capacity_mb);
    }
}
```

### Space Mesh Networking

```rust
use rotastellar_distributed::{SpaceMesh, MeshNode};
use rotastellar::Position;

fn main() {
    // Create mesh network
    let mut mesh = SpaceMesh::new();

    // Add nodes
    mesh.add_node(MeshNode::new("sat-001", Position::new(45.0, -122.0, 550.0)));
    mesh.add_node(MeshNode::new("sat-002", Position::new(46.0, -120.0, 550.0)));
    mesh.add_node(MeshNode::new("sat-003", Position::new(44.0, -118.0, 550.0)));

    // Add inter-satellite links
    mesh.add_link("sat-001", "sat-002", 1000.0, 2.0);  // bandwidth, latency
    mesh.add_link("sat-002", "sat-003", 1000.0, 2.5);

    // Find optimal route
    let route = mesh.find_route("sat-001", "sat-003").unwrap();
    println!("Route: {}", route.hops.join(" -> "));
    println!("Total latency: {:.1} ms", route.total_latency_ms);
}
```

## Features

- **Federated Learning** — Privacy-preserving distributed training across orbital nodes
- **Gradient Compression** — TopK, random sparsification, quantization for bandwidth-limited links
- **Model Partitioning** — Intelligent layer placement across heterogeneous nodes
- **Sync Scheduling** — Optimal ground station contact windows for data synchronization
- **Mesh Networking** — Dynamic routing for inter-satellite communication

## Links

- **Website:** https://rotastellar.com/products/distributed
- **Documentation:** https://docs.rs/rotastellar-distributed
- **Main SDK:** https://crates.io/crates/rotastellar

## Author

Created by [Subhadip Mitra](mailto:subhadipmitra@rotastellar.com) at [RotaStellar](https://rotastellar.com).

## License

MIT License — Copyright (c) 2026 RotaStellar
