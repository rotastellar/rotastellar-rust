//! Power Analysis for Orbital Compute
//!
//! Solar panel and battery sizing for orbital compute systems.

use serde::{Deserialize, Serialize};

/// Solar constant at 1 AU (W/m²)
const SOLAR_CONSTANT: f64 = 1361.0;

/// Types of solar cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarCellType {
    /// Standard silicon cells (~20% efficiency)
    Silicon,
    /// Triple-junction cells (~30% efficiency)
    TripleJunction,
    /// Perovskite cells (~25% efficiency)
    Perovskite,
}

impl std::fmt::Display for SolarCellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolarCellType::Silicon => write!(f, "silicon"),
            SolarCellType::TripleJunction => write!(f, "triple_junction"),
            SolarCellType::Perovskite => write!(f, "perovskite"),
        }
    }
}

impl SolarCellType {
    /// Get the typical efficiency for this cell type.
    pub fn efficiency(&self) -> f64 {
        match self {
            SolarCellType::Silicon => 0.20,
            SolarCellType::TripleJunction => 0.30,
            SolarCellType::Perovskite => 0.25,
        }
    }
}

/// Battery chemistry types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryChemistry {
    /// Standard Li-ion cells
    LithiumIon,
    /// Li-polymer cells
    LithiumPolymer,
    /// Nickel-hydrogen cells (long life)
    NickelHydrogen,
}

impl std::fmt::Display for BatteryChemistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryChemistry::LithiumIon => write!(f, "lithium_ion"),
            BatteryChemistry::LithiumPolymer => write!(f, "lithium_polymer"),
            BatteryChemistry::NickelHydrogen => write!(f, "nickel_hydrogen"),
        }
    }
}

impl BatteryChemistry {
    /// Get characteristics for this battery type.
    pub fn characteristics(&self) -> BatteryCharacteristics {
        match self {
            BatteryChemistry::LithiumIon => BatteryCharacteristics {
                specific_energy_wh_kg: 200.0,
                depth_of_discharge: 0.80,
                cycle_efficiency: 0.95,
                cycle_life: 5000,
            },
            BatteryChemistry::LithiumPolymer => BatteryCharacteristics {
                specific_energy_wh_kg: 180.0,
                depth_of_discharge: 0.70,
                cycle_efficiency: 0.93,
                cycle_life: 3000,
            },
            BatteryChemistry::NickelHydrogen => BatteryCharacteristics {
                specific_energy_wh_kg: 60.0,
                depth_of_discharge: 0.80,
                cycle_efficiency: 0.85,
                cycle_life: 50000,
            },
        }
    }
}

/// Battery characteristics.
#[derive(Debug, Clone, Copy)]
pub struct BatteryCharacteristics {
    /// Energy density (Wh/kg)
    pub specific_energy_wh_kg: f64,
    /// Depth of discharge (0.0-1.0)
    pub depth_of_discharge: f64,
    /// Round-trip efficiency (0.0-1.0)
    pub cycle_efficiency: f64,
    /// Number of cycles before significant degradation
    pub cycle_life: u32,
}

/// Power consumption profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerProfile {
    /// Average power consumption in watts
    pub average_power_w: f64,
    /// Peak power consumption in watts
    pub peak_power_w: Option<f64>,
    /// Idle power consumption in watts
    pub idle_power_w: Option<f64>,
    /// Duty cycle (0.0-1.0)
    pub duty_cycle: Option<f64>,
}

impl PowerProfile {
    /// Create a new power profile.
    pub fn new(average_power_w: f64) -> Self {
        Self {
            average_power_w,
            peak_power_w: None,
            idle_power_w: None,
            duty_cycle: None,
        }
    }

    /// Set peak power.
    pub fn with_peak_power(mut self, peak_w: f64) -> Self {
        self.peak_power_w = Some(peak_w);
        self
    }

    /// Set idle power.
    pub fn with_idle_power(mut self, idle_w: f64) -> Self {
        self.idle_power_w = Some(idle_w);
        self
    }
}

/// Solar panel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarConfig {
    /// Type of solar cells
    pub cell_type: SolarCellType,
    /// Cell efficiency (0.0-1.0)
    pub efficiency: f64,
    /// Annual degradation rate (0.0-1.0)
    pub degradation_per_year: f64,
    /// Panel area in m² (0 = auto-size)
    pub panel_area_m2: f64,
    /// Whether panels have sun tracking
    pub tracking: bool,
}

impl Default for SolarConfig {
    fn default() -> Self {
        Self {
            cell_type: SolarCellType::TripleJunction,
            efficiency: 0.30,
            degradation_per_year: 0.02,
            panel_area_m2: 0.0,
            tracking: false,
        }
    }
}

impl SolarConfig {
    /// Create a configuration for a specific cell type.
    pub fn for_cell_type(cell_type: SolarCellType) -> Self {
        Self {
            cell_type,
            efficiency: cell_type.efficiency(),
            ..Default::default()
        }
    }

    /// Set panel area.
    pub fn with_panel_area(mut self, area_m2: f64) -> Self {
        self.panel_area_m2 = area_m2;
        self
    }

    /// Enable tracking.
    pub fn with_tracking(mut self) -> Self {
        self.tracking = true;
        self
    }
}

/// Battery configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    /// Battery chemistry
    pub chemistry: BatteryChemistry,
    /// Capacity in Wh (0 = auto-size)
    pub capacity_wh: f64,
    /// Depth of discharge (0.0-1.0)
    pub depth_of_discharge: f64,
    /// Round-trip efficiency (0.0-1.0)
    pub cycle_efficiency: f64,
    /// Specific energy (Wh/kg)
    pub specific_energy_wh_kg: f64,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        let chars = BatteryChemistry::LithiumIon.characteristics();
        Self {
            chemistry: BatteryChemistry::LithiumIon,
            capacity_wh: 0.0,
            depth_of_discharge: chars.depth_of_discharge,
            cycle_efficiency: chars.cycle_efficiency,
            specific_energy_wh_kg: chars.specific_energy_wh_kg,
        }
    }
}

impl BatteryConfig {
    /// Create a configuration for a specific chemistry.
    pub fn for_chemistry(chemistry: BatteryChemistry) -> Self {
        let chars = chemistry.characteristics();
        Self {
            chemistry,
            capacity_wh: 0.0,
            depth_of_discharge: chars.depth_of_discharge,
            cycle_efficiency: chars.cycle_efficiency,
            specific_energy_wh_kg: chars.specific_energy_wh_kg,
        }
    }

    /// Set capacity.
    pub fn with_capacity(mut self, capacity_wh: f64) -> Self {
        self.capacity_wh = capacity_wh;
        self
    }
}

/// Complete power budget analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerBudget {
    /// Required power with margin in watts
    pub power_required_w: f64,
    /// Solar power generated in watts
    pub solar_power_generated_w: f64,
    /// Battery capacity in Wh
    pub battery_capacity_wh: f64,
    /// Solar panel area in m²
    pub solar_panel_area_m2: f64,
    /// Battery mass in kg
    pub battery_mass_kg: f64,
    /// Solar panel mass in kg
    pub solar_panel_mass_kg: f64,
    /// Eclipse duration in minutes
    pub eclipse_duration_min: f64,
    /// Whether power margin is positive
    pub positive_margin: bool,
    /// Power margin percentage
    pub margin_percent: f64,
    /// Warnings about the power budget
    pub warnings: Vec<String>,
}

/// Analyze power requirements for orbital compute systems.
///
/// # Example
///
/// ```rust
/// use rotastellar_compute::{PowerAnalyzer, PowerProfile};
///
/// let analyzer = PowerAnalyzer::new(550.0);
/// let profile = PowerProfile::new(500.0).with_peak_power(800.0);
/// let budget = analyzer.analyze(&profile, None, None, None, None);
/// println!("Solar panel area: {:.2} m²", budget.solar_panel_area_m2);
/// println!("Battery capacity: {:.1} Wh", budget.battery_capacity_wh);
/// ```
pub struct PowerAnalyzer {
    orbit_altitude_km: f64,
}

impl PowerAnalyzer {
    /// Solar panel specific power (W/kg)
    pub const SOLAR_PANEL_SPECIFIC_POWER: f64 = 100.0;
    /// Design margin (20%)
    pub const DESIGN_MARGIN: f64 = 0.2;

    /// Create a new power analyzer.
    pub fn new(orbit_altitude_km: f64) -> Self {
        Self { orbit_altitude_km }
    }

    /// Create an analyzer with default altitude (550 km).
    pub fn default_altitude() -> Self {
        Self::new(550.0)
    }

    /// Analyze power budget for a mission.
    pub fn analyze(
        &self,
        profile: &PowerProfile,
        solar_config: Option<&SolarConfig>,
        battery_config: Option<&BatteryConfig>,
        orbit_altitude_km: Option<f64>,
        mission_duration_years: Option<f64>,
    ) -> PowerBudget {
        let altitude = orbit_altitude_km.unwrap_or(self.orbit_altitude_km);
        let mission_years = mission_duration_years.unwrap_or(5.0);

        let solar = solar_config.cloned().unwrap_or_default();
        let battery = battery_config.cloned().unwrap_or_default();

        // Calculate orbital parameters
        let orbital_period_min = self.orbital_period(altitude);
        let eclipse_fraction = self.eclipse_fraction(altitude);
        let eclipse_duration = orbital_period_min * eclipse_fraction;
        let sunlight_duration = orbital_period_min * (1.0 - eclipse_fraction);

        // Power required with margin
        let power_required = profile.average_power_w * (1.0 + Self::DESIGN_MARGIN);

        // Account for degradation at EOL
        let eol_efficiency = solar.efficiency * (1.0 - solar.degradation_per_year * mission_years);

        // Required solar panel area
        let panel_area = if solar.panel_area_m2 > 0.0 {
            solar.panel_area_m2
        } else {
            let orbit_energy_wh = (power_required * orbital_period_min) / 60.0;
            let cosine_factor = if solar.tracking { 0.9 } else { 0.7 };
            let solar_power_needed = orbit_energy_wh / (sunlight_duration / 60.0);
            solar_power_needed / (SOLAR_CONSTANT * eol_efficiency * cosine_factor)
        };

        // Calculate actual solar power generated
        let cosine_factor = if solar.tracking { 0.9 } else { 0.7 };
        let solar_power = SOLAR_CONSTANT * panel_area * eol_efficiency * cosine_factor;

        // Battery sizing
        let eclipse_energy_wh = (power_required * eclipse_duration) / 60.0;
        let mut battery_capacity =
            eclipse_energy_wh / (battery.depth_of_discharge * battery.cycle_efficiency);
        battery_capacity *= 1.0 + Self::DESIGN_MARGIN;

        if battery.capacity_wh > 0.0 {
            battery_capacity = battery_capacity.max(battery.capacity_wh);
        }

        // Mass estimates
        let battery_mass = battery_capacity / battery.specific_energy_wh_kg;
        let solar_mass = solar_power / Self::SOLAR_PANEL_SPECIFIC_POWER;

        // Check margin
        let available_power = solar_power * (sunlight_duration / orbital_period_min);
        let margin_percent = ((available_power - power_required) / power_required) * 100.0;
        let positive_margin = margin_percent > 0.0;

        // Generate warnings
        let mut warnings = Vec::new();
        if !positive_margin {
            warnings.push("Negative power margin - increase solar panel area".to_string());
        }
        if battery_capacity > 1000.0 {
            warnings.push("Large battery capacity may impact mass budget".to_string());
        }
        if eol_efficiency < 0.2 {
            warnings.push("Significant solar cell degradation expected over mission life".to_string());
        }
        if eclipse_duration > 40.0 {
            warnings.push("Long eclipse duration - ensure adequate battery capacity".to_string());
        }

        PowerBudget {
            power_required_w: (power_required * 10.0).round() / 10.0,
            solar_power_generated_w: (solar_power * 10.0).round() / 10.0,
            battery_capacity_wh: (battery_capacity * 10.0).round() / 10.0,
            solar_panel_area_m2: (panel_area * 1000.0).round() / 1000.0,
            battery_mass_kg: (battery_mass * 100.0).round() / 100.0,
            solar_panel_mass_kg: (solar_mass * 100.0).round() / 100.0,
            eclipse_duration_min: (eclipse_duration * 10.0).round() / 10.0,
            positive_margin,
            margin_percent: (margin_percent * 10.0).round() / 10.0,
            warnings,
        }
    }

    /// Size solar panels for power requirement.
    pub fn size_solar_panels(
        &self,
        power_required_w: f64,
        orbit_altitude_km: Option<f64>,
        cell_type: Option<SolarCellType>,
        mission_years: Option<f64>,
    ) -> SolarPanelSizing {
        let altitude = orbit_altitude_km.unwrap_or(self.orbit_altitude_km);
        let cell = cell_type.unwrap_or(SolarCellType::TripleJunction);
        let years = mission_years.unwrap_or(5.0);

        let efficiency = cell.efficiency();
        let degradation = 0.02;

        let eol_efficiency = efficiency * (1.0 - degradation * years);
        let eclipse_fraction = self.eclipse_fraction(altitude);
        let sunlight_fraction = 1.0 - eclipse_fraction;

        let required_solar = (power_required_w / sunlight_fraction) * (1.0 + Self::DESIGN_MARGIN);
        let cosine_factor = 0.7;
        let panel_area = required_solar / (SOLAR_CONSTANT * eol_efficiency * cosine_factor);

        SolarPanelSizing {
            panel_area_m2: (panel_area * 1000.0).round() / 1000.0,
            cell_type: cell,
            bol_efficiency: efficiency,
            eol_efficiency: (eol_efficiency * 1000.0).round() / 1000.0,
            solar_power_w: (required_solar * 10.0).round() / 10.0,
            mass_estimate_kg: ((required_solar / Self::SOLAR_PANEL_SPECIFIC_POWER) * 100.0).round() / 100.0,
        }
    }

    /// Size battery for eclipse power.
    pub fn size_battery(
        &self,
        power_required_w: f64,
        orbit_altitude_km: Option<f64>,
        chemistry: Option<BatteryChemistry>,
    ) -> BatterySizing {
        let altitude = orbit_altitude_km.unwrap_or(self.orbit_altitude_km);
        let chem = chemistry.unwrap_or(BatteryChemistry::LithiumIon);
        let chars = chem.characteristics();

        let orbital_period = self.orbital_period(altitude);
        let eclipse_fraction = self.eclipse_fraction(altitude);
        let eclipse_min = orbital_period * eclipse_fraction;

        let eclipse_energy = (power_required_w * eclipse_min) / 60.0;
        let mut capacity = eclipse_energy / (chars.depth_of_discharge * chars.cycle_efficiency);
        capacity *= 1.0 + Self::DESIGN_MARGIN;

        let mass = capacity / chars.specific_energy_wh_kg;
        let orbits_per_day = (24.0 * 60.0) / orbital_period;
        let cycles_per_year = orbits_per_day * 365.0;

        BatterySizing {
            capacity_wh: (capacity * 10.0).round() / 10.0,
            chemistry: chem,
            mass_kg: (mass * 100.0).round() / 100.0,
            eclipse_duration_min: (eclipse_min * 10.0).round() / 10.0,
            depth_of_discharge: chars.depth_of_discharge,
            cycles_per_year: cycles_per_year.round() as u32,
            expected_life_years: ((chars.cycle_life as f64 / cycles_per_year) * 10.0).round() / 10.0,
        }
    }

    fn orbital_period(&self, altitude_km: f64) -> f64 {
        let earth_radius = 6371.0;
        let earth_mu = 398600.4418;
        let a = earth_radius + altitude_km;
        let period_s = 2.0 * std::f64::consts::PI * (a.powi(3) / earth_mu).sqrt();
        period_s / 60.0
    }

    fn eclipse_fraction(&self, altitude_km: f64) -> f64 {
        let earth_radius = 6371.0;
        let r = earth_radius + altitude_km;
        let sin_rho = earth_radius / r;
        sin_rho.asin() / std::f64::consts::PI
    }
}

impl Default for PowerAnalyzer {
    fn default() -> Self {
        Self::default_altitude()
    }
}

/// Solar panel sizing result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarPanelSizing {
    /// Required panel area in m²
    pub panel_area_m2: f64,
    /// Cell type used
    pub cell_type: SolarCellType,
    /// Beginning-of-life efficiency
    pub bol_efficiency: f64,
    /// End-of-life efficiency
    pub eol_efficiency: f64,
    /// Solar power generated in watts
    pub solar_power_w: f64,
    /// Estimated mass in kg
    pub mass_estimate_kg: f64,
}

/// Battery sizing result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatterySizing {
    /// Required capacity in Wh
    pub capacity_wh: f64,
    /// Battery chemistry
    pub chemistry: BatteryChemistry,
    /// Mass in kg
    pub mass_kg: f64,
    /// Eclipse duration in minutes
    pub eclipse_duration_min: f64,
    /// Depth of discharge
    pub depth_of_discharge: f64,
    /// Number of charge cycles per year
    pub cycles_per_year: u32,
    /// Expected battery life in years
    pub expected_life_years: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_analyzer() {
        let analyzer = PowerAnalyzer::new(550.0);
        let profile = PowerProfile::new(500.0);

        let budget = analyzer.analyze(&profile, None, None, None, None);

        assert!(budget.power_required_w > 500.0);
        assert!(budget.solar_panel_area_m2 > 0.0);
        assert!(budget.battery_capacity_wh > 0.0);
    }

    #[test]
    fn test_positive_margin_with_oversized_panels() {
        let analyzer = PowerAnalyzer::new(550.0);
        let profile = PowerProfile::new(300.0);
        // Oversize the panels to ensure positive margin
        let solar = SolarConfig::default().with_panel_area(5.0);

        let budget = analyzer.analyze(&profile, Some(&solar), None, None, None);

        assert!(budget.positive_margin);
        assert!(budget.margin_percent > 0.0);
    }

    #[test]
    fn test_size_solar_panels() {
        let analyzer = PowerAnalyzer::new(550.0);

        let sizing = analyzer.size_solar_panels(500.0, None, None, None);

        assert!(sizing.panel_area_m2 > 0.0);
        assert!(sizing.eol_efficiency < sizing.bol_efficiency);
        assert!(sizing.mass_estimate_kg > 0.0);
    }

    #[test]
    fn test_size_battery() {
        let analyzer = PowerAnalyzer::new(550.0);

        let sizing = analyzer.size_battery(500.0, None, None);

        assert!(sizing.capacity_wh > 0.0);
        assert!(sizing.mass_kg > 0.0);
        assert!(sizing.cycles_per_year > 5000);
        assert!(sizing.expected_life_years > 0.0);
    }

    #[test]
    fn test_different_cell_types() {
        let analyzer = PowerAnalyzer::new(550.0);

        let silicon = analyzer.size_solar_panels(500.0, None, Some(SolarCellType::Silicon), None);
        let triple = analyzer.size_solar_panels(500.0, None, Some(SolarCellType::TripleJunction), None);

        // Silicon needs more area due to lower efficiency
        assert!(silicon.panel_area_m2 > triple.panel_area_m2);
    }

    #[test]
    fn test_different_battery_chemistries() {
        let analyzer = PowerAnalyzer::new(550.0);

        let li_ion = analyzer.size_battery(500.0, None, Some(BatteryChemistry::LithiumIon));
        let ni_h2 = analyzer.size_battery(500.0, None, Some(BatteryChemistry::NickelHydrogen));

        // NiH2 has much longer cycle life
        assert!(ni_h2.expected_life_years > li_ion.expected_life_years);
        // But much lower energy density = more mass
        assert!(ni_h2.mass_kg > li_ion.mass_kg);
    }

    #[test]
    fn test_solar_config_with_tracking() {
        let analyzer = PowerAnalyzer::new(550.0);
        let profile = PowerProfile::new(500.0);

        let no_tracking = SolarConfig::default();
        let with_tracking = SolarConfig::default().with_tracking();

        let budget_no_track = analyzer.analyze(&profile, Some(&no_tracking), None, None, None);
        let budget_track = analyzer.analyze(&profile, Some(&with_tracking), None, None, None);

        // Tracking generates more power from same area, so needs smaller area
        assert!(budget_track.solar_panel_area_m2 < budget_no_track.solar_panel_area_m2);
    }

    #[test]
    fn test_warnings_for_high_power() {
        let analyzer = PowerAnalyzer::new(550.0);
        let profile = PowerProfile::new(2000.0);

        let budget = analyzer.analyze(&profile, None, None, None, None);

        assert!(budget.battery_capacity_wh > 1000.0);
        assert!(budget.warnings.iter().any(|w| w.contains("battery")));
    }
}
