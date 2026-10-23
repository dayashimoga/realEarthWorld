use realearth_geospatial::transforms::wgs84_to_ecef;
use realearth_ingestion::dem_grid::DemGrid;
use realearth_rendering::FloatingOriginCamera;
use realearth_shared::coordinates::Wgs84Coord;

/// Navigation mode for viewer camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    /// 6-DOF free flying camera.
    Fly,
    /// Ground-clamped walking camera adhering to DEM terrain elevation.
    Walk,
    /// Orbit camera rotating around a target landmark or geographic focal point.
    Orbit,
    /// Unconstrained free-look camera.
    Free,
    /// Third-person chase perspective.
    Chase,
}

/// Predefined real-world landmarks for instant navigation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Landmark {
    pub name: &'static str,
    pub coord: Wgs84Coord,
    pub yaw: f32,
    pub pitch: f32,
}

impl Landmark {
    pub const SAN_FRANCISCO_TRANSAMERICA: Self = Self {
        name: "Transamerica Pyramid (San Francisco)",
        coord: Wgs84Coord {
            lat: 37.7952,
            lon: -122.4028,
            alt: 35.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    pub const SAN_FRANCISCO_SALESFORCE: Self = Self {
        name: "Salesforce Tower (San Francisco)",
        coord: Wgs84Coord {
            lat: 37.7897,
            lon: -122.3972,
            alt: 45.0,
        },
        yaw: -1.2,
        pitch: -0.25,
    };

    pub const SAN_FRANCISCO_FERRY_BUILDING: Self = Self {
        name: "Ferry Building & Coastal Embarcadero (San Francisco)",
        coord: Wgs84Coord {
            lat: 37.7955,
            lon: -122.3937,
            alt: 12.0,
        },
        yaw: -2.8,
        pitch: -0.15,
    };

    pub const SAN_FRANCISCO_COIT_TOWER: Self = Self {
        name: "Coit Tower & Telegraph Hill (San Francisco)",
        coord: Wgs84Coord {
            lat: 37.8024,
            lon: -122.4058,
            alt: 95.0,
        },
        yaw: -0.8,
        pitch: -0.2,
    };

    pub const SAN_FRANCISCO_BAY_BRIDGE: Self = Self {
        name: "I-80 Bay Bridge Elevated Approach (San Francisco)",
        coord: Wgs84Coord {
            lat: 37.7885,
            lon: -122.3900,
            alt: 30.0,
        },
        yaw: -2.2,
        pitch: -0.18,
    };

    pub const SAN_FRANCISCO_BROADWAY_TUNNEL: Self = Self {
        name: "Broadway Tunnel under Russian Hill (San Francisco)",
        coord: Wgs84Coord {
            lat: 37.7980,
            lon: -122.4135,
            alt: 48.0,
        },
        yaw: -1.57,
        pitch: -0.15,
    };

    pub const BENGALURU_VIDHANA_SOUDHA: Self = Self {
        name: "Vidhana Soudha (Bengaluru Central)",
        coord: Wgs84Coord {
            lat: 12.9797,
            lon: 77.5907,
            alt: 940.0,
        },
        yaw: -1.57,
        pitch: -0.3,
    };

    pub const MYSURU_PALACE: Self = Self {
        name: "Mysore Palace & Chamundi View (Mysuru Urban)",
        coord: Wgs84Coord {
            lat: 12.3052,
            lon: 76.6552,
            alt: 780.0,
        },
        yaw: -1.57,
        pitch: -0.25,
    };

    pub const CHENNAI_CENTRAL: Self = Self {
        name: "Chennai Central Station & Marina (Chennai)",
        coord: Wgs84Coord {
            lat: 13.0827,
            lon: 80.2755,
            alt: 35.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    pub const HYDERABAD_CHARMINAR: Self = Self {
        name: "Charminar & Mecca Masjid (Hyderabad Central)",
        coord: Wgs84Coord {
            lat: 17.3616,
            lon: 78.4747,
            alt: 565.0,
        },
        yaw: -1.57,
        pitch: -0.25,
    };

    pub const CORRIDOR_KAVERI_BRIDGE: Self = Self {
        name: "NH 275 Kaveri River Bridge (Srirangapatna)",
        coord: Wgs84Coord {
            lat: 12.4230,
            lon: 76.6950,
            alt: 740.0,
        },
        yaw: -1.2,
        pitch: -0.15,
    };

    pub const CORRIDOR_PALAR_BRIDGE: Self = Self {
        name: "NH 48 Palar River Bridge (Ranipet / Vellore)",
        coord: Wgs84Coord {
            lat: 12.9240,
            lon: 79.1350,
            alt: 240.0,
        },
        yaw: -1.57,
        pitch: -0.15,
    };

    pub const CORRIDOR_PVNR_EXPRESSWAY: Self = Self {
        name: "PVNR Elevated Expressway (Hyderabad Corridor)",
        coord: Wgs84Coord {
            lat: 17.3500,
            lon: 78.4400,
            alt: 570.0,
        },
        yaw: -1.4,
        pitch: -0.18,
    };

    pub const NEW_YORK_ONE_WTC: Self = Self {
        name: "One World Trade Center & Lower Manhattan (New York)",
        coord: Wgs84Coord {
            lat: 40.7128,
            lon: -74.0134,
            alt: 55.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    pub const LOS_ANGELES_WILSHIRE_GRAND: Self = Self {
        name: "Wilshire Grand & Downtown Skyline (Los Angeles)",
        coord: Wgs84Coord {
            lat: 34.0500,
            lon: -118.2600,
            alt: 120.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    pub const CHICAGO_WILLIS_TOWER: Self = Self {
        name: "Willis Tower & Chicago River Loop (Chicago)",
        coord: Wgs84Coord {
            lat: 41.8789,
            lon: -87.6359,
            alt: 210.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    pub const SEATTLE_SPACE_NEEDLE: Self = Self {
        name: "Space Needle & Elliott Bay (Seattle)",
        coord: Wgs84Coord {
            lat: 47.6205,
            lon: -122.3493,
            alt: 75.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    pub const MIAMI_BRICKELL_PANORAMA: Self = Self {
        name: "Panorama Tower & Biscayne Bay (Miami)",
        coord: Wgs84Coord {
            lat: 25.7628,
            lon: -80.1912,
            alt: 25.0,
        },
        yaw: -1.57,
        pitch: -0.2,
    };

    /// Creates a camera bookmark dynamically for any US city definition.
    pub fn from_us_city(city: &realearth_shared::us_catalog::UsCityDefinition) -> Self {
        Self {
            name: city.anchor_landmark_name,
            coord: Wgs84Coord {
                lat: city.anchor.lat,
                lon: city.anchor.lon,
                alt: city.anchor.alt + (city.anchor_height_m as f64 * 0.20).max(25.0),
            },
            yaw: city.anchor_yaw,
            pitch: city.anchor_pitch,
        }
    }
}

/// Interactive camera controller.
#[derive(Debug, Clone)]
pub struct CameraController {
    pub mode: CameraMode,
    pub coord: Wgs84Coord,
    pub default_coord: Wgs84Coord,
    pub default_yaw: f32,
    pub default_pitch: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fly_speed: f64,
    pub speed_multiplier: f64,
    pub walk_eye_height: f32,
    // Orbit camera parameters
    pub orbit_target: Wgs84Coord,
    pub orbit_radius: f64,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
}

impl CameraController {
    pub fn new(initial_coord: Wgs84Coord) -> Self {
        Self {
            mode: CameraMode::Fly,
            coord: initial_coord,
            default_coord: initial_coord,
            default_yaw: -std::f32::consts::FRAC_PI_2,
            default_pitch: -0.25,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.25,
            fly_speed: 60.0, // meters per second baseline
            speed_multiplier: 1.0,
            walk_eye_height: 1.8, // standard eye level
            orbit_target: initial_coord,
            orbit_radius: 180.0,
            orbit_yaw: -std::f32::consts::FRAC_PI_2,
            orbit_pitch: -0.28,
        }
    }

    /// Teleports camera to specified landmark and updates reset bookmarks.
    pub fn jump_to(&mut self, landmark: &Landmark) {
        self.coord = landmark.coord;
        self.yaw = landmark.yaw;
        self.pitch = landmark.pitch;
        self.default_coord = landmark.coord;
        self.default_yaw = landmark.yaw;
        self.default_pitch = landmark.pitch;
        self.orbit_target = landmark.coord;
        self.orbit_yaw = landmark.yaw;
        self.orbit_pitch = landmark.pitch;
    }

    /// Resets camera position, orientation, and speed back to the anchor landmark.
    pub fn reset_to_default(&mut self) {
        self.coord = self.default_coord;
        self.yaw = self.default_yaw;
        self.pitch = self.default_pitch;
        self.speed_multiplier = 1.0;
        self.orbit_target = self.default_coord;
        self.orbit_yaw = self.default_yaw;
        self.orbit_pitch = self.default_pitch;
        self.orbit_radius = 180.0;
    }

    /// Cycles through available navigation modes: Fly -> Walk -> Orbit -> Free -> Chase -> Fly.
    pub fn cycle_mode(&mut self) -> CameraMode {
        self.mode = match self.mode {
            CameraMode::Fly => CameraMode::Walk,
            CameraMode::Walk => CameraMode::Orbit,
            CameraMode::Orbit => CameraMode::Free,
            CameraMode::Free => CameraMode::Chase,
            CameraMode::Chase => CameraMode::Fly,
        };
        self.mode
    }

    /// Increases flight movement speed multiplier (capped at 8.0x).
    pub fn increase_speed(&mut self) -> f64 {
        self.speed_multiplier = (self.speed_multiplier * 1.5).min(8.0);
        self.effective_speed()
    }

    /// Decreases flight movement speed multiplier (floored at 0.15x).
    pub fn decrease_speed(&mut self) -> f64 {
        self.speed_multiplier = (self.speed_multiplier / 1.5).max(0.15);
        self.effective_speed()
    }

    /// Calculates current effective speed in meters per second.
    pub fn effective_speed(&self) -> f64 {
        self.fly_speed * self.speed_multiplier
    }

    /// Clamps camera altitude above ground DEM elevation to prevent terrain clipping.
    pub fn clamp_terrain(&mut self, dem: &DemGrid) {
        let ground = dem.sample_bilinear(&self.coord);
        let min_clearance = match self.mode {
            CameraMode::Walk => self.walk_eye_height as f64,
            _ => 1.5, // 1.5m clearance to prevent clipping below ground in Fly/Free/Orbit
        };
        if self.coord.alt < (ground as f64 + min_clearance) {
            self.coord.alt = ground as f64 + min_clearance;
        }
    }

    /// Rotates the orbit camera around the focal landmark target.
    pub fn orbit_rotate(&mut self, delta_yaw: f32, delta_pitch: f32, dem: Option<&DemGrid>) {
        self.orbit_yaw += delta_yaw;
        self.orbit_pitch = (self.orbit_pitch + delta_pitch).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.08,
            -0.04, // keep viewing target from level or above
        );
        self.recompute_orbit_position(dem);
    }

    /// Adjusts orbit distance from focal target.
    pub fn orbit_zoom(&mut self, delta_radius: f64, dem: Option<&DemGrid>) {
        self.orbit_radius = (self.orbit_radius + delta_radius).clamp(15.0, 5000.0);
        self.recompute_orbit_position(dem);
    }

    /// Recomputes camera geographic coordinates from orbit radius, yaw, and pitch.
    pub fn recompute_orbit_position(&mut self, dem: Option<&DemGrid>) {
        let m_per_deg_lat = 111_000.0;
        let m_per_deg_lon = 111_320.0 * (self.orbit_target.lat.to_radians()).cos().abs().max(0.1);

        let horiz_dist = self.orbit_radius * (self.orbit_pitch.cos() as f64).abs();
        let vert_dist = -self.orbit_radius * (self.orbit_pitch.sin() as f64);

        let d_lat = -(self.orbit_yaw.sin() as f64 * horiz_dist) / m_per_deg_lat;
        let d_lon = -(self.orbit_yaw.cos() as f64 * horiz_dist) / m_per_deg_lon;

        self.coord.lat = self.orbit_target.lat + d_lat;
        self.coord.lon = self.orbit_target.lon + d_lon;
        self.coord.alt = (self.orbit_target.alt + vert_dist).max(2.0);

        self.yaw = self.orbit_yaw;
        self.pitch = self.orbit_pitch;

        if let Some(dem) = dem {
            self.clamp_terrain(dem);
        }
    }

    /// Computes the forward 3D velocity vector in meters per second for streaming lookahead.
    pub fn forward_velocity_vector(&self, speed_mps: f64) -> glam::Vec3 {
        let forward_x = self.yaw.cos() * self.pitch.cos();
        let forward_y = self.pitch.sin();
        let forward_z = self.yaw.sin() * self.pitch.cos();
        glam::Vec3::new(
            forward_x * speed_mps as f32,
            forward_y * speed_mps as f32,
            forward_z * speed_mps as f32,
        )
    }

    /// Moves camera forward/backward along view heading.
    pub fn move_forward(&mut self, amount: f64, dem: Option<&DemGrid>) {
        if self.mode == CameraMode::Orbit {
            self.orbit_zoom(-amount * 2.0, dem);
            return;
        }

        let forward_x = (self.yaw.cos() * self.pitch.cos()) as f64;
        let forward_z = (self.yaw.sin() * self.pitch.cos()) as f64;

        // Approx 111,000 meters per degree lat, 88,000 meters per degree lon at lat 38
        let m_per_deg_lat = 111_000.0;
        let m_per_deg_lon = 111_320.0 * (self.coord.lat.to_radians()).cos().abs().max(0.1);

        let d_lat = (forward_z * amount) / m_per_deg_lat;
        let d_lon = (forward_x * amount) / m_per_deg_lon;

        self.coord.lat += d_lat;
        self.coord.lon += d_lon;

        if self.mode == CameraMode::Walk {
            if let Some(dem) = dem {
                let ground = dem.sample_bilinear(&self.coord);
                self.coord.alt = (ground + self.walk_eye_height) as f64;
            }
        } else {
            let forward_y = self.pitch.sin() as f64;
            self.coord.alt += forward_y * amount;
            if let Some(dem) = dem {
                self.clamp_terrain(dem);
            }
        }
    }

    /// Strafes camera left/right perpendicular to view heading.
    pub fn strafe(&mut self, amount: f64, dem: Option<&DemGrid>) {
        if self.mode == CameraMode::Orbit {
            self.orbit_rotate((amount * 0.005) as f32, 0.0, dem);
            return;
        }

        let right_x = (-self.yaw.sin()) as f64;
        let right_z = (self.yaw.cos()) as f64;

        let m_per_deg_lat = 111_000.0;
        let m_per_deg_lon = 111_320.0 * (self.coord.lat.to_radians()).cos().abs().max(0.1);

        let d_lat = (right_z * amount) / m_per_deg_lat;
        let d_lon = (right_x * amount) / m_per_deg_lon;

        self.coord.lat += d_lat;
        self.coord.lon += d_lon;

        if self.mode == CameraMode::Walk {
            if let Some(dem) = dem {
                let ground = dem.sample_bilinear(&self.coord);
                self.coord.alt = (ground + self.walk_eye_height) as f64;
            }
        } else if let Some(dem) = dem {
            self.clamp_terrain(dem);
        }
    }

    /// Moves camera vertically (Fly/Free mode only, or orbit tilt in Orbit mode).
    pub fn move_vertical(&mut self, amount: f64) {
        if self.mode == CameraMode::Fly || self.mode == CameraMode::Free {
            self.coord.alt = (self.coord.alt + amount).max(1.0);
        } else if self.mode == CameraMode::Orbit {
            self.orbit_rotate(0.0, (amount * 0.005) as f32, None);
        }
    }

    /// Rotates camera pitch and yaw.
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        if self.mode == CameraMode::Orbit {
            self.orbit_rotate(delta_yaw, delta_pitch, None);
            return;
        }

        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.05,
            std::f32::consts::FRAC_PI_2 - 0.05,
        );
    }

    /// Synchronizes internal coordinates to the rendering camera.
    pub fn update_camera(&self, camera: &mut FloatingOriginCamera) {
        camera.pos_ecef = wgs84_to_ecef(self.coord);
        camera.yaw = self.yaw;
        camera.pitch = self.pitch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use realearth_ingestion::dem_grid::DemGrid;

    #[test]
    fn test_camera_mode_cycling() {
        let mut ctrl = CameraController::new(Wgs84Coord::new(37.795, -122.405, 30.0));
        assert_eq!(ctrl.mode, CameraMode::Fly);

        assert_eq!(ctrl.cycle_mode(), CameraMode::Walk);
        assert_eq!(ctrl.cycle_mode(), CameraMode::Orbit);
        assert_eq!(ctrl.cycle_mode(), CameraMode::Free);
        assert_eq!(ctrl.cycle_mode(), CameraMode::Chase);
        assert_eq!(ctrl.cycle_mode(), CameraMode::Fly);
    }

    #[test]
    fn test_camera_speed_controls() {
        let mut ctrl = CameraController::new(Wgs84Coord::new(37.795, -122.405, 30.0));
        let baseline = ctrl.effective_speed();
        assert_eq!(baseline, 60.0);

        let faster = ctrl.increase_speed();
        assert_eq!(faster, 90.0);
        assert!(ctrl.speed_multiplier > 1.0);

        let slower = ctrl.decrease_speed();
        assert_eq!(slower, 60.0);

        let very_slow = ctrl.decrease_speed();
        assert!(very_slow < 60.0);
    }

    #[test]
    fn test_terrain_collision_clamping() {
        let mut ctrl = CameraController::new(Wgs84Coord::new(37.795, -122.405, 30.0));
        let dem = DemGrid::create_san_francisco_coastal_dem(32, 32);

        // Force camera way below ground
        ctrl.coord.alt = -100.0;
        ctrl.clamp_terrain(&dem);

        let ground = dem.sample_bilinear(&ctrl.coord);
        assert!(
            ctrl.coord.alt >= ground as f64 + 1.0,
            "Camera altitude ({}) must be clamped above ground ({})",
            ctrl.coord.alt,
            ground
        );
    }

    #[test]
    fn test_camera_reset_to_default() {
        let mut ctrl = CameraController::new(Wgs84Coord::new(37.795, -122.405, 30.0));
        ctrl.coord.lat += 0.05;
        ctrl.coord.lon -= 0.05;
        ctrl.coord.alt += 500.0;
        ctrl.yaw = 1.0;
        ctrl.pitch = 0.5;
        ctrl.increase_speed();

        ctrl.reset_to_default();
        assert_eq!(ctrl.coord, ctrl.default_coord);
        assert_eq!(ctrl.yaw, ctrl.default_yaw);
        assert_eq!(ctrl.pitch, ctrl.default_pitch);
        assert_eq!(ctrl.speed_multiplier, 1.0);
    }

    #[test]
    fn test_orbit_camera_navigation() {
        let mut ctrl = CameraController::new(Wgs84Coord::new(37.795, -122.405, 30.0));
        ctrl.mode = CameraMode::Orbit;
        ctrl.orbit_radius = 200.0;
        ctrl.recompute_orbit_position(None);

        let initial_alt = ctrl.coord.alt;
        assert!(initial_alt > 0.0);

        // Zooming in reduces orbit radius
        ctrl.orbit_zoom(-50.0, None);
        assert_eq!(ctrl.orbit_radius, 150.0);

        // Rotating orbit yaw changes coordinates
        let prev_lat = ctrl.coord.lat;
        ctrl.orbit_rotate(0.5, 0.0, None);
        assert_ne!(ctrl.coord.lat, prev_lat);
    }

    #[test]
    fn test_update_camera_sync() {
        let ctrl = CameraController::new(Wgs84Coord::new(37.795, -122.405, 30.0));
        let mut cam = FloatingOriginCamera::new(ctrl.coord, 1.77);
        ctrl.update_camera(&mut cam);

        assert_eq!(cam.yaw, ctrl.yaw);
        assert_eq!(cam.pitch, ctrl.pitch);
    }
}
