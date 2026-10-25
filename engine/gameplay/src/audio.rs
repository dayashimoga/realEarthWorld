//! Pure Rust real-time vehicle audio synthesis and sound effects engine.
//!
//! Provides mathematically authentic acoustic models for engine RPM harmonics,
//! throttle load rumble, turbo whine, tire scrub and skid squeal, brake friction,
//! collision impact impulses, aerodynamic wind ambience, traffic horns, and UI chimes.
//!
//! Runs completely safely in headless test environments without requiring OS sound servers.

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Sound profile preset matched to vehicle category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleAudioPreset {
    /// High-revving V8/V10 with aggressive intake resonance and exhaust crackle.
    SportsCoupe,
    /// Smooth, refined inline-6 / V6 with acoustic damping.
    LuxurySedan,
    /// Torquey V8 with heavy low-end rumble and all-terrain tire tread hum.
    OverlandSuv,
    /// Commercial 4-cylinder / V6 with balanced urban noise isolation.
    CityTaxi,
    /// Heavy turbocharged inline-6 diesel with low-frequency rumble and turbo whistle.
    HeavyTruck,
    /// Commercial mid-mount turbo-diesel with pneumatic brake hiss.
    TransitBus,
    /// Screaming high-revving 4-cylinder superbike (up to 12,000 RPM).
    Superbike,
    /// Silent electric powertrain with high-frequency inverter whine.
    Electric,
}

impl VehicleAudioPreset {
    pub fn cylinders(&self) -> f32 {
        match self {
            Self::SportsCoupe => 8.0,
            Self::LuxurySedan => 6.0,
            Self::OverlandSuv => 8.0,
            Self::CityTaxi => 4.0,
            Self::HeavyTruck => 6.0,
            Self::TransitBus => 6.0,
            Self::Superbike => 4.0,
            Self::Electric => 0.0,
        }
    }

    pub fn idle_rpm(&self) -> f32 {
        match self {
            Self::Superbike => 1400.0,
            Self::SportsCoupe => 850.0,
            Self::LuxurySedan => 650.0,
            Self::OverlandSuv => 700.0,
            Self::CityTaxi => 750.0,
            Self::HeavyTruck => 550.0,
            Self::TransitBus => 600.0,
            Self::Electric => 0.0,
        }
    }

    pub fn redline_rpm(&self) -> f32 {
        match self {
            Self::Superbike => 12500.0,
            Self::SportsCoupe => 8200.0,
            Self::LuxurySedan => 6800.0,
            Self::OverlandSuv => 6200.0,
            Self::CityTaxi => 6000.0,
            Self::HeavyTruck => 2400.0,
            Self::TransitBus => 2800.0,
            Self::Electric => 14000.0,
        }
    }
}

/// Instantaneous UI sound trigger events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiSoundCue {
    JobAccepted,
    WaypointReached,
    PassengerBoarded,
    JobCompletedPayout,
    MenuNavigate,
    MenuSelect,
    CollisionWarning,
}

/// Instantaneous telemetry state driving audio synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct AudioTelemetry {
    pub engine_rpm: f32,
    pub throttle: f32,
    pub speed_kph: f32,
    pub brake_input: f32,
    pub tire_slip_ratio: f32,
    pub tire_slip_angle_deg: f32,
    pub collision_impulse: f32,
    pub horn_pressed: bool,
}

/// Dynamic sound synthesis mixer telemetry and decibel levels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioMixerOutput {
    pub engine_gain: f32,
    pub tire_squeal_gain: f32,
    pub brake_squeal_gain: f32,
    pub wind_gain: f32,
    pub collision_gain: f32,
    pub horn_gain: f32,
    pub dominant_freq_hz: f32,
}

/// Pure Rust real-time acoustic synthesizer.
pub struct VehicleAudioEngine {
    pub preset: VehicleAudioPreset,
    pub master_volume: f32,
    pub engine_volume: f32,
    pub sfx_volume: f32,
    pub ambience_volume: f32,
    pub ui_volume: f32,

    // Phase accumulators for continuous synthesis
    phase_engine: f32,
    phase_turbo: f32,
    phase_wind: f32,
    phase_tire: f32,
    phase_brake: f32,
    phase_horn: f32,
    phase_ui: f32,

    // Transient triggers
    collision_decay: f32,
    active_ui_cue: Option<(UiSoundCue, f32)>, // (cue, progress_seconds)
    last_telemetry: AudioTelemetry,
    noise_seed: u32,
}

impl Default for VehicleAudioEngine {
    fn default() -> Self {
        Self::new(VehicleAudioPreset::SportsCoupe)
    }
}

impl VehicleAudioEngine {
    pub fn new(preset: VehicleAudioPreset) -> Self {
        Self {
            preset,
            master_volume: 0.85,
            engine_volume: 0.90,
            sfx_volume: 0.80,
            ambience_volume: 0.60,
            ui_volume: 0.95,
            phase_engine: 0.0,
            phase_turbo: 0.0,
            phase_wind: 0.0,
            phase_tire: 0.0,
            phase_brake: 0.0,
            phase_horn: 0.0,
            phase_ui: 0.0,
            collision_decay: 0.0,
            active_ui_cue: None,
            last_telemetry: AudioTelemetry::default(),
            noise_seed: 0x12345678,
        }
    }

    /// Triggers an immediate UI acoustic chime.
    pub fn play_ui_cue(&mut self, cue: UiSoundCue) {
        self.active_ui_cue = Some((cue, 0.0));
        self.phase_ui = 0.0;
    }

    /// Updates dynamic vehicle telemetry parameters and steps acoustic decay timers.
    pub fn update_telemetry(&mut self, telemetry: AudioTelemetry, dt: f32) -> AudioMixerOutput {
        self.last_telemetry = telemetry;

        // Trigger collision transient if high impulse detected
        if telemetry.collision_impulse > 2.0 {
            self.collision_decay = (telemetry.collision_impulse * 0.15).clamp(0.5, 1.2);
        } else if self.collision_decay > 0.0 {
            self.collision_decay = (self.collision_decay - dt * 3.5).max(0.0);
        }

        // Advance UI cue timer
        if let Some((cue, ref mut time)) = self.active_ui_cue {
            *time += dt;
            let duration = match cue {
                UiSoundCue::JobCompletedPayout => 1.2,
                UiSoundCue::PassengerBoarded => 0.8,
                UiSoundCue::JobAccepted => 0.6,
                UiSoundCue::WaypointReached => 0.4,
                _ => 0.25,
            };
            if *time > duration {
                self.active_ui_cue = None;
            }
        }

        // Compute dominant frequency and gains
        let cylinders = self.preset.cylinders();
        let dominant_freq_hz = if cylinders > 0.0 {
            (telemetry.engine_rpm / 60.0) * (cylinders * 0.5)
        } else {
            (telemetry.speed_kph * 15.0).clamp(100.0, 3500.0) // Electric motor pitch
        };

        let throttle_load = 0.35 + telemetry.throttle.clamp(0.0, 1.0) * 0.65;
        let rpm_frac =
            (telemetry.engine_rpm / self.preset.redline_rpm().max(1000.0)).clamp(0.0, 1.2);
        let engine_gain =
            (0.2 + rpm_frac * 0.8) * throttle_load * self.engine_volume * self.master_volume;

        // Tire squeal gain: triggered by high slip ratio (>0.12) or high slip angle (>5.0 deg)
        let slip_severity = (telemetry.tire_slip_ratio.abs() - 0.10).max(0.0) * 3.0
            + (telemetry.tire_slip_angle_deg.abs() - 4.0).max(0.0) * 0.15;
        let tire_squeal_gain = (slip_severity * (telemetry.speed_kph / 25.0).clamp(0.0, 1.0))
            .clamp(0.0, 1.0)
            * self.sfx_volume
            * self.master_volume;

        // Brake squeal gain: high brake pressure at moving speed
        let brake_squeal_gain = if telemetry.brake_input > 0.45 && telemetry.speed_kph > 8.0 {
            ((telemetry.brake_input - 0.45) * 1.8).clamp(0.0, 0.7)
                * (telemetry.speed_kph / 40.0).clamp(0.0, 1.0)
                * self.sfx_volume
                * self.master_volume
        } else {
            0.0
        };

        // Aerodynamic wind roar gain (proportional to v^2)
        let speed_ratio = (telemetry.speed_kph / 120.0).clamp(0.0, 2.5);
        let wind_gain = (speed_ratio * speed_ratio * 0.45).clamp(0.0, 0.9)
            * self.ambience_volume
            * self.master_volume;

        let collision_gain =
            (self.collision_decay * self.sfx_volume * self.master_volume).clamp(0.0, 1.0);
        let horn_gain = if telemetry.horn_pressed {
            0.85 * self.sfx_volume * self.master_volume
        } else {
            0.0
        };

        AudioMixerOutput {
            engine_gain,
            tire_squeal_gain,
            brake_squeal_gain,
            wind_gain,
            collision_gain,
            horn_gain,
            dominant_freq_hz,
        }
    }

    /// Fast pseudo-random noise generator for stochastic acoustic textures (wind, tire scrub).
    fn next_noise(&mut self) -> f32 {
        self.noise_seed = self
            .noise_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        ((self.noise_seed >> 9) as f32 / 8388607.0) - 1.0
    }

    /// Generates interleaved stereo 16-bit PCM audio samples at the specified sample rate.
    pub fn generate_pcm_samples(&mut self, num_frames: usize, sample_rate: u32) -> Vec<i16> {
        let sr = sample_rate as f32;
        let mut buffer = Vec::with_capacity(num_frames * 2);
        let dt = 1.0 / sr;

        let cylinders = self.preset.cylinders();
        let f0 = if cylinders > 0.0 {
            (self.last_telemetry.engine_rpm / 60.0) * (cylinders * 0.5)
        } else {
            (self.last_telemetry.speed_kph * 15.0).clamp(100.0, 3500.0)
        };

        let mixer = self.update_telemetry(self.last_telemetry, dt * (num_frames as f32));

        for _ in 0..num_frames {
            // 1. Engine Synthesis (Fundamental + Harmonics + Throttle Rumble)
            let engine_sample = if cylinders > 0.0 {
                let wave_1 = (self.phase_engine * 2.0 * PI).sin();
                let wave_2 = (self.phase_engine * 4.0 * PI).sin() * 0.55;
                let wave_3 = (self.phase_engine * 6.0 * PI).sin() * 0.35;
                let wave_4 = (self.phase_engine * 8.0 * PI).sin() * 0.20;
                let sub_bass =
                    (self.phase_engine * PI).sin() * (0.25 + self.last_telemetry.throttle * 0.4);

                // Throttle saturation
                let raw_engine = wave_1 + wave_2 + wave_3 + wave_4 + sub_bass;
                let mut sample = (raw_engine * 0.6).tanh() * mixer.engine_gain;
                self.phase_engine = (self.phase_engine + f0 * dt).fract();

                // Turbo whine at high throttle/RPM
                if self.last_telemetry.throttle > 0.5 && self.last_telemetry.engine_rpm > 3000.0 {
                    let turbo_f = f0 * 6.2;
                    let turbo_sample = (self.phase_turbo * 2.0 * PI).sin()
                        * (self.last_telemetry.throttle - 0.5)
                        * 0.15;
                    sample += turbo_sample * mixer.engine_gain;
                    self.phase_turbo = (self.phase_turbo + turbo_f * dt).fract();
                }
                sample
            } else {
                // Electric high-efficiency motor whine
                let motor_f = (self.last_telemetry.speed_kph * 24.0 + 120.0).clamp(120.0, 4800.0);
                let sample = (self.phase_engine * 2.0 * PI).sin() * 0.45 * mixer.engine_gain;
                self.phase_engine = (self.phase_engine + motor_f * dt).fract();
                sample
            };

            // 2. Tire Scrub and Skid Squeal (Dual-band modulated noise)
            let mut tire_sample = 0.0f32;
            if mixer.tire_squeal_gain > 0.01 {
                let squeal_mod = (self.phase_tire * 2.0 * PI * 850.0).sin() * 0.6
                    + (self.phase_tire * 2.0 * PI * 1850.0).sin() * 0.4;
                let noise = self.next_noise();
                tire_sample = (squeal_mod * 0.7 + noise * 0.3) * mixer.tire_squeal_gain;
                self.phase_tire = (self.phase_tire + dt).fract();
            }

            // 3. Brake Friction Squeal
            let mut brake_sample = 0.0f32;
            if mixer.brake_squeal_gain > 0.01 {
                let squeal = (self.phase_brake * 2.0 * PI * 2450.0).sin();
                brake_sample = squeal * mixer.brake_squeal_gain;
                self.phase_brake = (self.phase_brake + dt).fract();
            }

            // 4. Wind Ambience with aerodynamic turbulence
            let turbulence = (self.phase_wind * 2.0 * PI * 1.5).sin() * 0.2 + 0.8;
            let wind_sample = self.next_noise() * 0.5 * turbulence * mixer.wind_gain;
            self.phase_wind = (self.phase_wind + dt).fract();

            // 5. Collision Impact
            let mut collision_sample = 0.0f32;
            if mixer.collision_gain > 0.01 {
                let impact_boom = (self.collision_decay * 45.0 * 2.0 * PI).sin();
                collision_sample =
                    (impact_boom * 0.7 + self.next_noise() * 0.3) * mixer.collision_gain;
            }

            // 6. Horn Dual-Tone (370 Hz + 440 Hz)
            let mut horn_sample = 0.0f32;
            if mixer.horn_gain > 0.01 {
                let h1 = (self.phase_horn * 2.0 * PI * 370.0).sin() * 0.5;
                let h2 = (self.phase_horn * 2.0 * PI * 440.0).sin() * 0.5;
                horn_sample = (h1 + h2) * mixer.horn_gain;
                self.phase_horn = (self.phase_horn + dt).fract();
            }

            // 7. UI Chime Synthesis
            let mut ui_sample = 0.0f32;
            if let Some((cue, time)) = self.active_ui_cue {
                let ui_freq = match cue {
                    UiSoundCue::JobAccepted => 523.25 + time * 600.0, // Rising C5 -> E5
                    UiSoundCue::WaypointReached => 659.25,            // E5 ping
                    UiSoundCue::PassengerBoarded => 440.0 + (time * 12.0).sin() * 50.0, // Dual chime
                    UiSoundCue::JobCompletedPayout => 523.25 + (time * 8.0).floor() * 110.0, // Major arpeggio
                    UiSoundCue::MenuNavigate => 400.0,
                    UiSoundCue::MenuSelect => 600.0,
                    UiSoundCue::CollisionWarning => 880.0 + (time * 20.0).sin() * 200.0,
                };
                let env = (1.0 - (time / 1.0)).clamp(0.0, 1.0);
                ui_sample = (self.phase_ui * 2.0 * PI * ui_freq).sin()
                    * env
                    * self.ui_volume
                    * self.master_volume
                    * 0.65;
                self.phase_ui = (self.phase_ui + dt).fract();
            }

            // Sum mixed channels with master limiter
            let left_mix = (engine_sample * 0.95
                + tire_sample * 0.85
                + brake_sample * 0.7
                + wind_sample * 0.6
                + collision_sample
                + horn_sample * 0.9
                + ui_sample)
                .clamp(-1.0, 1.0);
            let right_mix = (engine_sample * 0.95
                + tire_sample * 0.85
                + brake_sample * 0.7
                + wind_sample * 0.6
                + collision_sample
                + horn_sample * 0.9
                + ui_sample)
                .clamp(-1.0, 1.0);

            buffer.push((left_mix * 32767.0) as i16);
            buffer.push((right_mix * 32767.0) as i16);
        }

        buffer
    }

    /// Exports raw generated PCM audio into a standard RIFF/WAV byte buffer for testing or inspection.
    pub fn export_wav_bytes(&mut self, num_frames: usize, sample_rate: u32) -> Vec<u8> {
        let pcm = self.generate_pcm_samples(num_frames, sample_rate);
        let num_channels = 2u16;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * (num_channels as u32) * ((bits_per_sample / 8) as u32);
        let block_align = num_channels * (bits_per_sample / 8);
        let data_size = (pcm.len() * 2) as u32;

        let mut wav = Vec::with_capacity(44 + pcm.len() * 2);
        // RIFF chunk
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt subchunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format = 1
        wav.extend_from_slice(&num_channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data subchunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        for sample in pcm {
            wav.extend_from_slice(&sample.to_le_bytes());
        }

        wav
    }
}
