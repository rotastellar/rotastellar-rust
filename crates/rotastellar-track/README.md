# rotastellar-track

**High-Performance Satellite Tracking & TLE Propagation**

SGP4/SDP4 propagation and satellite position calculation optimized for batch processing.

## Installation

```toml
[dependencies]
rotastellar-track = "0.1"
```

## Quick Start

### TLE Parsing and Propagation

```rust
use rotastellar_track::{TLE, Propagator};
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse TLE
    let tle = TLE::parse(&[
        "ISS (ZARYA)",
        "1 25544U 98067A   24001.50000000  .00016717  00000-0  10270-3 0  9025",
        "2 25544  51.6400 208.9163 0006703  40.5765  35.4667 15.49560927421258",
    ])?;

    // Create propagator
    let propagator = Propagator::new(&tle)?;

    // Get position at specific time
    let now = Utc::now();
    let state = propagator.propagate(now)?;

    println!("Position (ECI): x={:.1}km, y={:.1}km, z={:.1}km",
             state.position.x, state.position.y, state.position.z);
    println!("Velocity: {:.3} km/s", state.velocity_magnitude());

    // Convert to geodetic coordinates
    let geo = state.to_geodetic();
    println!("Lat: {:.2}°, Lon: {:.2}°, Alt: {:.1}km",
             geo.latitude, geo.longitude, geo.altitude_km);

    Ok(())
}
```

### Batch Propagation

```rust
use rotastellar_track::{TLE, BatchPropagator};
use chrono::{Utc, Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load multiple TLEs
    let tles = vec![
        TLE::parse(&iss_lines)?,
        TLE::parse(&starlink_lines)?,
        TLE::parse(&debris_lines)?,
    ];

    // Create batch propagator (uses SIMD where available)
    let propagator = BatchPropagator::new(&tles)?;

    // Propagate all satellites at once
    let now = Utc::now();
    let states = propagator.propagate_all(now)?;

    for (i, state) in states.iter().enumerate() {
        let geo = state.to_geodetic();
        println!("{}: {:.2}°, {:.2}°", tles[i].name, geo.latitude, geo.longitude);
    }

    // Generate position time series
    let start = Utc::now();
    let end = start + Duration::hours(2);
    let step = Duration::minutes(1);

    let trajectory = propagator.trajectory(0, start, end, step)?;
    println!("Generated {} position points", trajectory.len());

    Ok(())
}
```

### Pass Prediction

```rust
use rotastellar_track::{TLE, Propagator, Observer};
use chrono::{Utc, Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tle = TLE::parse(&iss_lines)?;
    let propagator = Propagator::new(&tle)?;

    // Observer at Kennedy Space Center
    let observer = Observer::new(28.5729, -80.6490, 0.0);

    // Find passes in next 24 hours
    let start = Utc::now();
    let end = start + Duration::hours(24);
    let min_elevation = 10.0;  // degrees

    let passes = propagator.find_passes(&observer, start, end, min_elevation)?;

    for pass in passes {
        println!("AOS: {}", pass.aos);
        println!("  Max elevation: {:.1}° at {}", pass.max_elevation, pass.tca);
        println!("  LOS: {}", pass.los);
        println!("  Duration: {} seconds", pass.duration_seconds);
    }

    Ok(())
}
```

## Why Rust?

Satellite tracking requires:
- **High-precision math** — Orbital mechanics needs accurate double-precision calculations
- **Real-time processing** — Tracking thousands of objects simultaneously
- **Low latency** — Conjunction warnings must be computed quickly
- **Batch efficiency** — SIMD optimization for catalog-wide propagation

Rust delivers the performance needed for production tracking systems while maintaining memory safety.

## Features

- **TLE Parsing** — Full Two-Line Element set support with validation
- **SGP4/SDP4 Propagation** — NORAD-standard orbit propagation algorithms
- **Batch Processing** — SIMD-optimized multi-satellite propagation
- **Pass Prediction** — Find satellite visibility windows from any location
- **Coordinate Transforms** — ECI, ECEF, geodetic, topocentric conversions

## Links

- **Website:** https://rotastellar.com/products/orbital-intelligence
- **Documentation:** https://docs.rs/rotastellar-track
- **Main SDK:** https://crates.io/crates/rotastellar

## Author

Created by [Subhadip Mitra](mailto:subhadipmitra@rotastellar.com) at [RotaStellar](https://rotastellar.com).

## License

MIT License — Copyright (c) 2026 RotaStellar
