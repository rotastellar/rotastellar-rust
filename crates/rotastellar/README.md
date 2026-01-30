# rotastellar

**Rust SDK for RotaStellar — Space Computing Infrastructure**

Core types, utilities, and client for the RotaStellar platform.

## Installation

```toml
[dependencies]
rotastellar = "0.1"
```

## Quick Start

```rust
use rotastellar::{Position, Orbit, Satellite, TimeRange};
use chrono::{Utc, Duration};

fn main() {
    // Create a geographic position (e.g., Kennedy Space Center)
    let ksc = Position::new(28.5729, -80.6490, 0.0);
    println!("KSC: {}°N, {}°W", ksc.latitude, ksc.longitude);

    // Define an ISS-like orbit using Keplerian elements
    let orbit = Orbit::new(
        6778.0,   // semi-major axis (km)
        0.0001,   // eccentricity
        51.6,     // inclination (deg)
        100.0,    // RAAN (deg)
        90.0,     // argument of periapsis (deg)
        0.0,      // true anomaly (deg)
    );
    println!("Orbital period: {:.1} minutes", orbit.period_minutes());
    println!("Apogee: {:.1} km, Perigee: {:.1} km", orbit.apogee_km(), orbit.perigee_km());

    // Create a satellite
    let satellite = Satellite::new(
        "ISS",
        "International Space Station",
        25544,
        Position::new(45.0, -122.0, 408.0),
        orbit,
    );

    // Define a time range
    let now = Utc::now();
    let range = TimeRange::new(now, now + Duration::hours(24));
    println!("Duration: {:.1} hours", range.duration_hours());
}
```

## Features

- **Position** — Geographic coordinates with altitude
- **Orbit** — Keplerian orbital elements with derived properties (period, apogee, perigee, velocity)
- **Satellite** — Satellite metadata and state
- **TimeRange** — Time window specifications
- **Validation** — Compile-time and runtime parameter validation

## Related Crates

| Crate | Description |
|-------|-------------|
| [rotastellar-intel](https://crates.io/crates/rotastellar-intel) | Satellite tracking, TLE parsing, conjunction analysis |
| [rotastellar-compute](https://crates.io/crates/rotastellar-compute) | Thermal, power, latency, and feasibility analysis |
| [rotastellar-distributed](https://crates.io/crates/rotastellar-distributed) | Federated learning, model partitioning, mesh routing |
| [rotastellar-track](https://crates.io/crates/rotastellar-track) | High-performance SGP4 propagation |

## Links

- **Website:** https://rotastellar.com
- **Documentation:** https://docs.rs/rotastellar
- **GitHub:** https://github.com/rotastellar/rotastellar-rust

## Author

Created by [Subhadip Mitra](mailto:subhadipmitra@rotastellar.com) at [RotaStellar](https://rotastellar.com).

## License

MIT License — Copyright (c) 2026 RotaStellar
