//! Simulation clock, time progression, and atmospheric weather states.

use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};

/// High-precision simulation clock resource.
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationClock {
    /// Current time of day in hours [0.0, 24.0). Default: 12.0 (Noon).
    pub time_hours: f64,
    /// Day of the year (1 to 365). Default: 80 (Spring Equinox).
    pub day_of_year: u32,
    /// Time acceleration multiplier (1.0 = real-time, 60.0 = 1 simulated hour per real minute).
    pub time_scale: f64,
    /// Total simulated ticks elapsed.
    pub total_ticks: u64,
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self {
            time_hours: 12.0, // Default to high noon
            day_of_year: 80,
            time_scale: 1.0,
            total_ticks: 0,
        }
    }
}

impl SimulationClock {
    /// Advances clock by `delta_seconds` of real time.
    pub fn advance(&mut self, delta_seconds: f64) {
        let hours_advanced = (delta_seconds * self.time_scale) / 3600.0;
        self.time_hours += hours_advanced;
        while self.time_hours >= 24.0 {
            self.time_hours -= 24.0;
            self.day_of_year = (self.day_of_year % 365) + 1;
        }
        self.total_ticks += 1;
    }
}

/// Dynamic weather classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    Clear,
    Overcast,
    Rain,
    Fog,
}

/// Atmospheric and lighting parameters derived from weather and time.
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtmosphericState {
    pub weather: WeatherType,
    pub fog_density: f32,
    pub ambient_light: [f32; 3],
    pub cloud_cover: f32,
    pub rain_intensity: f32,
}

impl Default for AtmosphericState {
    fn default() -> Self {
        Self {
            weather: WeatherType::Clear,
            fog_density: 0.0002,
            ambient_light: [0.2, 0.22, 0.25],
            cloud_cover: 0.1,
            rain_intensity: 0.0,
        }
    }
}

impl AtmosphericState {
    pub fn set_weather(&mut self, weather: WeatherType) {
        self.weather = weather;
        match weather {
            WeatherType::Clear => {
                self.fog_density = 0.0001;
                self.ambient_light = [0.25, 0.25, 0.28];
                self.cloud_cover = 0.1;
                self.rain_intensity = 0.0;
            }
            WeatherType::Overcast => {
                self.fog_density = 0.0005;
                self.ambient_light = [0.18, 0.18, 0.20];
                self.cloud_cover = 0.85;
                self.rain_intensity = 0.0;
            }
            WeatherType::Rain => {
                self.fog_density = 0.0015;
                self.ambient_light = [0.12, 0.13, 0.15];
                self.cloud_cover = 0.95;
                self.rain_intensity = 0.8;
            }
            WeatherType::Fog => {
                self.fog_density = 0.0080;
                self.ambient_light = [0.30, 0.30, 0.32];
                self.cloud_cover = 0.70;
                self.rain_intensity = 0.0;
            }
        }
    }
}
