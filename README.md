<p align="center">
  <img src="assets/logo-dark.jpg" alt="RotaStellar" width="400">
</p>

<p align="center">
  <strong>Rust SDK for Space Computing Infrastructure</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/rotastellar"><img src="https://img.shields.io/crates/v/rotastellar?color=orange&label=rotastellar" alt="crates.io"></a>
  <a href="https://crates.io/crates/rotastellar-compute"><img src="https://img.shields.io/crates/v/rotastellar-compute?color=orange&label=compute" alt="crates.io"></a>
  <a href="https://crates.io/crates/rotastellar-intel"><img src="https://img.shields.io/crates/v/rotastellar-intel?color=orange&label=intel" alt="crates.io"></a>
  <a href="https://crates.io/crates/rotastellar-distributed"><img src="https://img.shields.io/crates/v/rotastellar-distributed?color=orange&label=distributed" alt="crates.io"></a>
</p>

<p align="center">
  <a href="https://github.com/rotastellar/rotastellar-rust/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70+-orange" alt="Rust"></a>
  <a href="https://docs.rs/rotastellar"><img src="https://img.shields.io/badge/docs-docs.rs-blue" alt="Documentation"></a>
</p>

---

Plan, simulate, and operate orbital data centers and space intelligence systems.

## Crates

| Crate | Description |
|-------|-------------|
| [rotastellar](./crates/rotastellar) | Core types — Position, Orbit, Satellite, TimeRange |
| [rotastellar-compute](./crates/rotastellar-compute) | Feasibility, thermal, power, and latency analysis |
| [rotastellar-intel](./crates/rotastellar-intel) | Satellite tracking, TLE parsing, conjunction analysis |
| [rotastellar-distributed](./crates/rotastellar-distributed) | Federated learning, model partitioning, mesh routing |
| [rotastellar-track](./crates/rotastellar-track) | High-performance SGP4/SDP4 propagation |

## Installation

```toml
[dependencies]
rotastellar = "0.1"
rotastellar-compute = "0.1"
rotastellar-intel = "0.1"
rotastellar-distributed = "0.1"
```

## Quick Start

```rust
use rotastellar::{Position, Orbit};
use rotastellar_compute::{FeasibilityCalculator, WorkloadProfile, WorkloadType};

fn main() {
    // Define a position
    let ksc = Position::new(28.5729, -80.6490, 0.0);

    // Analyze workload feasibility
    let calc = FeasibilityCalculator::new(550.0);
    let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0)
        .with_memory_gb(32.0);

    let result = calc.analyze(&profile, None);
    println!("Feasible: {}, Rating: {:?}", result.feasible, result.rating);
}
```

## Why Rust?

Space computing requires:
- **Performance** — Processing thousands of satellites in real-time
- **Precision** — High-precision orbital mechanics calculations
- **Reliability** — Mission-critical systems can't crash
- **WASM** — Browser-based simulation and visualization tools

## Links

- **Website:** https://rotastellar.com
- **Documentation:** https://docs.rs/rotastellar
- **Python SDK:** https://github.com/rotastellar/rotastellar-python
- **Node.js SDK:** https://github.com/rotastellar/rotastellar-node

## Author

Created by [Subhadip Mitra](mailto:subhadipmitra@rotastellar.com) at [RotaStellar](https://rotastellar.com).

## License

MIT License — Copyright (c) 2026 RotaStellar
