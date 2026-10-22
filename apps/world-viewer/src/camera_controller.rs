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
}

/// Interactive camera controller.
#[derive(Debug, Clone)]
pub struct CameraController {
    pub mode: CameraMode,
    pub coord: Wgs84Coord,
    pub yaw: f32,
    pub pitch: f32,
    pub fly_speed: f64,
    pub walk_eye_height: f32,
}

impl CameraController {
    pub fn new(initial_coord: Wgs84Coord) -> Self {
        Self {
            mode: CameraMode::Fly,
            coord: initial_coord,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.25,
            fly_speed: 60.0,      // meters per second
            walk_eye_height: 1.8, // standard eye level
        }
    }

    /// Teleports camera to specified landmark.
    pub fn jump_to(&mut self, landmark: &Landmark) {
        self.coord = landmark.coord;
        self.yaw = landmark.yaw;
        self.pitch = landmark.pitch;
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
        }
    }

    /// Strafes camera left/right perpendicular to view heading.
    pub fn strafe(&mut self, amount: f64, dem: Option<&DemGrid>) {
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
        }
    }

    /// Moves camera vertically (Fly mode only).
    pub fn move_vertical(&mut self, amount: f64) {
        if self.mode == CameraMode::Fly {
            self.coord.alt = (self.coord.alt + amount).max(1.0);
        }
    }

    /// Rotates camera pitch and yaw.
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
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
        camera.maybe_rebase_origin(2000.0);
    }
}
