//! Real Earth Platform — Native Mobile C FFI & JNI Bindings.
//!
//! Exposes verified C ABI and JNI entry points for Android (cdylib: `librealearth.so`)
//! and iOS (staticlib: `librealearth.a`).

#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_float, c_int};
use std::sync::Mutex;

use realearth_shared::coordinates::Wgs84Coord;
use realearth_vehicles::{DynamicVehicle, FlatGround, VehicleConfig, VehicleInputs};

/// C-compatible telemetry struct passed across FFI boundary to Swift/Java.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RealEarthTelemetry {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub speed_kph: f32,
    pub engine_rpm: f32,
    pub gear: i32,
    pub suspension_fl: f32,
    pub suspension_fr: f32,
    pub suspension_rl: f32,
    pub suspension_rr: f32,
    pub is_active: i32,
}

impl Default for RealEarthTelemetry {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            alt: 0.0,
            speed_kph: 0.0,
            engine_rpm: 800.0,
            gear: 1,
            suspension_fl: 0.0,
            suspension_fr: 0.0,
            suspension_rl: 0.0,
            suspension_rr: 0.0,
            is_active: 0,
        }
    }
}

struct EngineContext {
    vehicle: DynamicVehicle,
    anchor: Wgs84Coord,
    ground_query: FlatGround,
}

static ENGINE_STATE: Mutex<Option<EngineContext>> = Mutex::new(None);

/// Initializes the Real Earth mobile simulation engine.
/// Returns 0 on success, -1 on invalid parameter, -2 on internal error.
#[no_mangle]
pub extern "C" fn realearth_init(region_str: *const c_char) -> c_int {
    if region_str.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(region_str) };
    let region = match c_str.to_str() {
        Ok(s) => s.to_lowercase(),
        Err(_) => return -1,
    };

    let anchor = if region.contains("nyc") || region.contains("new-york") {
        Wgs84Coord::new(40.7128, -74.0134, 10.0) // One WTC
    } else if region.contains("la") || region.contains("los-angeles") {
        Wgs84Coord::new(34.0500, -118.2600, 85.0) // Wilshire Grand
    } else if region.contains("chicago") {
        Wgs84Coord::new(41.8789, -87.6359, 180.0) // Willis Tower
    } else if region.contains("seattle") {
        Wgs84Coord::new(47.6205, -122.3493, 65.0) // Space Needle
    } else if region.contains("miami") {
        Wgs84Coord::new(25.7628, -80.1912, 4.0) // Brickell
    } else if region.contains("bengaluru") || region == "in" {
        Wgs84Coord::new(12.9797, 77.5907, 920.0) // Vidhana Soudha
    } else {
        Wgs84Coord::new(37.7952, -122.4028, 10.0) // San Francisco Transamerica
    };

    let ground_query = FlatGround::new(0.0);
    let mut vehicle = DynamicVehicle::new(VehicleConfig::default(), glam::Vec3::ZERO, -1.57);
    vehicle.recover_on_surface(0.0);

    let mut lock = match ENGINE_STATE.lock() {
        Ok(guard) => guard,
        Err(_) => return -2,
    };

    *lock = Some(EngineContext {
        vehicle,
        anchor,
        ground_query,
    });

    0
}

/// Steps the physics simulation and vehicle dynamics by delta_time seconds.
#[no_mangle]
pub extern "C" fn realearth_step(
    delta_time: c_float,
    throttle: c_float,
    steer: c_float,
    brake: c_float,
) -> c_int {
    let mut lock = match ENGINE_STATE.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };

    if let Some(ref mut ctx) = *lock {
        let inputs = VehicleInputs {
            throttle: throttle.clamp(0.0, 1.0),
            steer: steer.clamp(-1.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            handbrake: false,
            ..Default::default()
        };
        ctx.vehicle
            .step(&inputs, &ctx.ground_query, delta_time.max(0.001));
        0
    } else {
        -2
    }
}

/// Retrieves the current vehicle and planetary telemetry.
#[no_mangle]
pub extern "C" fn realearth_get_telemetry(out_telem: *mut RealEarthTelemetry) -> c_int {
    if out_telem.is_null() {
        return -1;
    }

    let lock = match ENGINE_STATE.lock() {
        Ok(guard) => guard,
        Err(_) => return -1,
    };

    if let Some(ref ctx) = *lock {
        let m_per_deg_lat = 111_320.0;
        let m_per_deg_lon = 111_320.0 * ctx.anchor.lat.to_radians().cos().abs().max(0.01);

        let lat = ctx.anchor.lat + (ctx.vehicle.position.z as f64 / m_per_deg_lat);
        let lon = ctx.anchor.lon + (ctx.vehicle.position.x as f64 / m_per_deg_lon);
        let alt = ctx.anchor.alt + ctx.vehicle.position.y as f64;

        unsafe {
            *out_telem = RealEarthTelemetry {
                lat,
                lon,
                alt,
                speed_kph: ctx.vehicle.speed_kph,
                engine_rpm: ctx.vehicle.engine_rpm,
                gear: ctx.vehicle.current_gear as i32,
                suspension_fl: ctx.vehicle.wheels[0].compression,
                suspension_fr: ctx.vehicle.wheels[1].compression,
                suspension_rl: ctx.vehicle.wheels[2].compression,
                suspension_rr: ctx.vehicle.wheels[3].compression,
                is_active: 1,
            };
        }
        0
    } else {
        unsafe {
            *out_telem = RealEarthTelemetry::default();
        }
        -2
    }
}

/// Shuts down the engine and frees all allocated resources.
#[no_mangle]
pub extern "C" fn realearth_shutdown() -> c_int {
    if let Ok(mut lock) = ENGINE_STATE.lock() {
        *lock = None;
        0
    } else {
        -1
    }
}

// =========================================================================
// Android JNI Bindings (com.realearth.world.RealEarthEngine)
// =========================================================================

#[no_mangle]
pub unsafe extern "C" fn Java_com_realearth_world_RealEarthEngine_nativeInit(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    region_jstr: *const c_char,
) -> c_int {
    realearth_init(region_jstr)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_realearth_world_RealEarthEngine_nativeStep(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    dt: c_float,
    throttle: c_float,
    steer: c_float,
    brake: c_float,
) -> c_int {
    realearth_step(dt, throttle, steer, brake)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_realearth_world_RealEarthEngine_nativeShutdown(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) -> c_int {
    realearth_shutdown()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_c_ffi_lifecycle() {
        let region = CString::new("new-york").unwrap();
        assert_eq!(realearth_init(region.as_ptr()), 0);

        for _ in 0..10 {
            assert_eq!(realearth_step(1.0 / 60.0, 0.8, 0.0, 0.0), 0);
        }

        let mut telem = RealEarthTelemetry::default();
        assert_eq!(realearth_get_telemetry(&mut telem), 0);
        assert_eq!(telem.is_active, 1);
        assert!(telem.speed_kph > 0.0);
        assert!((telem.lat - 40.7128).abs() < 0.01);

        assert_eq!(realearth_shutdown(), 0);
    }
}
