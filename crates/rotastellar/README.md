# rotastellar

**Rust SDK for RotaStellar - Space Computing Infrastructure**

Plan, simulate, and operate orbital data centers and space intelligence systems.

🚀 **Launching Q1 2026**

## Installation

```toml
[dependencies]
rotastellar = "0.0.1"
```

## Overview

RotaStellar provides tools for:

- **Orbital Compute Suite** — Plan and simulate space-based data centers
- **Orbital Intelligence** — Track, analyze, and monitor orbital activity

## Coming Soon

```rust
use rotastellar::OrbitalIntel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OrbitalIntel::new("api_key")?;

    // Track any satellite
    let iss = client.satellite("ISS").await?;
    let pos = iss.position().await?;
    println!("ISS: {}, {}", pos.lat, pos.lon);

    Ok(())
}
```

## Related Crates

- [rotastellar-compute](https://crates.io/crates/rotastellar-compute) — Orbital compute tools
- [rotastellar-intel](https://crates.io/crates/rotastellar-intel) — Orbital intelligence
- [rotastellar-track](https://crates.io/crates/rotastellar-track) — Satellite tracking

## Links

- **Website:** https://rotastellar.com
- **Documentation:** https://rotastellar.com/docs
- **GitHub:** https://github.com/rotastellar/rotastellar-rust
- **docs.rs:** https://docs.rs/rotastellar

## Part of Rota, Inc.

- [rotalabs.ai](https://rotalabs.ai) — Trust Intelligence Research
- [rotascale.com](https://rotascale.com) — Enterprise AI & Data

## License

MIT License — Copyright (c) 2026 Rota, Inc.
