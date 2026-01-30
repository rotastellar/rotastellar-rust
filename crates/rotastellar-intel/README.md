# rotastellar-intel

**Orbital Intelligence & Space Situational Awareness**

Track satellites, parse TLEs, analyze conjunctions, and detect orbital patterns.

## Installation

```toml
[dependencies]
rotastellar-intel = "0.1"
```

## Quick Start

### TLE Parsing

```rust
use rotastellar_intel::TLE;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a Two-Line Element set
    let tle_lines = [
        "ISS (ZARYA)",
        "1 25544U 98067A   24001.50000000  .00016717  00000-0  10270-3 0  9025",
        "2 25544  51.6400 208.9163 0006703  40.5765  35.4667 15.49560927421258",
    ];

    let tle = TLE::parse(&tle_lines)?;
    println!("Satellite: {}", tle.name);
    println!("NORAD ID: {}", tle.norad_id);
    println!("Inclination: {:.2}°", tle.inclination_deg);
    println!("Period: {:.2} minutes", tle.orbital_period_minutes);

    // Get position at epoch
    let position = tle.propagate(None)?;
    println!("Position: {:.2}°, {:.2}°", position.latitude, position.longitude);

    Ok(())
}
```

### Satellite Tracking

```rust
use rotastellar_intel::{Tracker, GroundStation};
use rotastellar::Position;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a tracker
    let mut tracker = Tracker::new();
    tracker.add_tle("ISS", &tle)?;

    // Get current position
    let pos = tracker.get_position("ISS")?;

    // Calculate passes over a ground station
    let gs = GroundStation::new(
        "KSC",
        Position::new(28.5729, -80.6490, 0.0),
        10.0,  // min elevation
    );

    let passes = tracker.predict_passes("ISS", &gs, 24)?;  // 24 hours
    for p in passes {
        println!("AOS: {}, Max El: {:.1}°", p.aos, p.max_elevation_deg);
    }

    Ok(())
}
```

### Conjunction Analysis

```rust
use rotastellar_intel::{ConjunctionAnalyzer, RiskLevel};

fn main() {
    let analyzer = ConjunctionAnalyzer::new();

    // Analyze collision probability
    let conjunction = analyzer.analyze(
        "ISS",
        "DEBRIS-12345",
        0.5,   // miss distance km
        10.0,  // relative velocity km/s
    );

    println!("Risk Level: {:?}", conjunction.risk_level);
    println!("Collision Probability: {:.2e}", conjunction.collision_probability);

    if conjunction.risk_level == RiskLevel::Critical {
        println!("⚠️  Maneuver recommended!");
    }
}
```

### Pattern Detection

```rust
use rotastellar_intel::{PatternDetector, PatternType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let detector = PatternDetector::new();

    // Detect maneuvers from TLE history
    let patterns = detector.detect("STARLINK-1234", 30)?;  // 30 days

    for pattern in patterns {
        match pattern.pattern_type {
            PatternType::OrbitRaise => {
                println!("Orbit raise detected: +{:.1} km", pattern.delta_altitude_km);
            }
            PatternType::Maneuver => {
                println!("Maneuver: Δv = {:.2} m/s", pattern.delta_v_m_s);
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Features

- **TLE Parsing** — Full Two-Line Element support with SGP4 propagation
- **Satellite Tracking** — Real-time position and pass prediction
- **Conjunction Analysis** — Collision probability using NASA CARA methodology
- **Pattern Detection** — Maneuver detection, anomaly identification

## Links

- **Website:** https://rotastellar.com/products/orbital-intelligence
- **Documentation:** https://docs.rs/rotastellar-intel
- **Main SDK:** https://crates.io/crates/rotastellar

## Author

Created by [Subhadip Mitra](mailto:subhadipmitra@rotastellar.com) at [RotaStellar](https://rotastellar.com).

## License

MIT License — Copyright (c) 2026 RotaStellar
