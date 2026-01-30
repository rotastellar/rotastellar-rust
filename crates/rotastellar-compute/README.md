# rotastellar-compute

**Orbital Compute Planning & Simulation**

Feasibility analysis, thermal simulation, power budgeting, and latency modeling for space-based computing.

## Installation

```toml
[dependencies]
rotastellar-compute = "0.1"
```

## Quick Start

### Feasibility Analysis

```rust
use rotastellar_compute::{FeasibilityCalculator, WorkloadProfile, WorkloadType};

fn main() {
    // Create a calculator for 550km altitude
    let calc = FeasibilityCalculator::new(550.0);

    // Define your workload
    let profile = WorkloadProfile::new(WorkloadType::Inference, 10.0)
        .with_memory_gb(32.0)
        .with_latency_requirement_ms(100.0);

    // Analyze feasibility
    let result = calc.analyze(&profile, None);
    println!("Feasible: {}", result.feasible);
    println!("Rating: {:?}", result.rating);  // Excellent, Good, Marginal, or NotFeasible
    println!("Thermal margin: {:.1}%", result.thermal_margin_percent);
    println!("Power margin: {:.1}%", result.power_margin_percent);
}
```

### Thermal Simulation

```rust
use rotastellar_compute::{ThermalSimulator, ThermalConfig, ThermalEnvironment};

fn main() {
    // Create simulator
    let sim = ThermalSimulator::new();

    // Configure for 500W heat dissipation
    let config = ThermalConfig::for_power(500.0);

    // LEO environment at 550km
    let env = ThermalEnvironment::leo(550.0);

    // Run simulation
    let result = sim.simulate(&config, &env);
    println!("Equilibrium temperature: {:.1}°C", result.equilibrium_temp_c);
    println!("Max temperature: {:.1}°C", result.max_temp_c);
    println!("Radiator area required: {:.2} m²", result.radiator_sizing.area_m2);
}
```

### Power Analysis

```rust
use rotastellar_compute::{PowerAnalyzer, PowerProfile, SolarConfig, BatteryConfig};

fn main() {
    // Analyzer for 550km orbit
    let analyzer = PowerAnalyzer::new(550.0);

    // Power requirements
    let profile = PowerProfile::new(500.0);  // 500W average

    // Analyze with default configs
    let budget = analyzer.analyze(&profile, None, None, None, None);
    println!("Solar panel area: {:.2} m²", budget.solar_panel_area_m2);
    println!("Battery capacity: {:.0} Wh", budget.battery_capacity_wh);
    println!("Eclipse duration: {:.1} minutes", budget.eclipse_duration_min);
}
```

### Latency Modeling

```rust
use rotastellar_compute::LatencySimulator;

fn main() {
    // Simulator for 550km altitude
    let sim = LatencySimulator::new(550.0);

    // Simulate with 100ms processing time
    let result = sim.simulate(Some(100.0), None);
    println!("Propagation delay: {:.1} ms", result.propagation_delay_ms);
    println!("Processing time: {:.1} ms", result.processing_time_ms);
    println!("Total latency: {:.1} ms", result.total_latency_ms);

    // Compare different altitudes
    let altitudes = [400.0, 550.0, 800.0, 1200.0];
    let comparison = sim.compare_altitudes(&altitudes);
    for alt_result in comparison {
        println!("{}km: {:.1}ms", alt_result.altitude_km, alt_result.typical_latency_ms);
    }
}
```

## Features

- **Feasibility Analysis** — Evaluate workload suitability for orbital compute
- **Thermal Simulation** — Model heat rejection using Stefan-Boltzmann law
- **Power Analysis** — Solar panel and battery sizing for orbital systems
- **Latency Modeling** — End-to-end latency for space-ground communication

## Links

- **Website:** https://rotastellar.com/products/compute
- **Documentation:** https://docs.rs/rotastellar-compute
- **Main SDK:** https://crates.io/crates/rotastellar

## Author

Created by [Subhadip Mitra](mailto:subhadipmitra@rotastellar.com) at [RotaStellar](https://rotastellar.com).

## License

MIT License — Copyright (c) 2026 RotaStellar
