# rotastellar-distributed

Distributed compute coordination for Earth-space AI workloads.

**Status:** Coming Q1 2026

## Overview

`rotastellar-distributed` enables AI training and inference across hybrid Earth-space infrastructure. Coordinate federated learning, partition models optimally, and synchronize through bandwidth-constrained orbital links.

## Installation

```toml
[dependencies]
rotastellar-distributed = "0.0.1"
```

## Features

### Federated Learning

```rust
use rotastellar_distributed::{FederatedClient, CompressionConfig, CompressionMethod};

let compression = CompressionConfig::new()
    .method(CompressionMethod::TopKQuantized)
    .k_ratio(0.01)
    .quantization_bits(8);

let client = FederatedClient::new("orbital-3", compression);
let gradients = client.train_step(&model, &batch);
client.sync(gradients, Priority::High);
```

### Model Partitioning

```rust
use rotastellar_distributed::{PartitionOptimizer, ModelProfile};

let profile = ModelProfile::from_onnx("model.onnx");
let optimizer = PartitionOptimizer::new();
let partition = optimizer.optimize(&profile, &topology);
```

### Sync Scheduler

```rust
use rotastellar_distributed::{SyncScheduler, GroundStation, Priority};

let mut scheduler = SyncScheduler::new();
scheduler.add_station(GroundStation::new("svalbard", 78.2, 15.6));
scheduler.schedule_sync("orbital-1", 50_000_000, Priority::Critical);
```

### Space Mesh

```rust
use rotastellar_distributed::SpaceMesh;

let mut mesh = SpaceMesh::new();
mesh.add_node("orbital-1", 550.0);
mesh.add_node("orbital-2", 550.0);
let route = mesh.find_route("orbital-1", "ground-svalbard");
```

## Documentation

Full documentation: https://docs.rs/rotastellar-distributed

## Links

- Website: https://rotastellar.com/products/distributed-compute
- Interactive Demo: https://rotastellar.com/products/distributed-compute/demo
- Research: https://rotastellar.com/research

## License

MIT License
