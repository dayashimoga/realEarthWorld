#ifndef REALEARTH_H
#define REALEARTH_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

/**
 * Real Earth Telemetry structure passed over C ABI.
 */
typedef struct {
    double lat;
    double lon;
    double alt;
    float speed_kph;
    float engine_rpm;
    int32_t gear;
    float suspension_fl;
    float suspension_fr;
    float suspension_rl;
    float suspension_rr;
    int32_t is_active;
} RealEarthTelemetry;

/**
 * Initializes the Real Earth simulation engine for the target geographic region.
 * @param region Name of the region ("new-york", "los-angeles", "chicago", "seattle", "miami", "san-francisco", "bengaluru")
 * @return 0 on success, non-zero on error.
 */
int32_t realearth_init(const char* region);

/**
 * Steps the physics simulation and vehicle dynamics by delta_time seconds.
 * @param delta_time Simulation time step in seconds (e.g. 0.016667 for 60 FPS)
 * @param throttle Throttle input [0.0, 1.0]
 * @param steer Steering input [-1.0, 1.0]
 * @param brake Braking input [0.0, 1.0]
 * @return 0 on success, non-zero on error.
 */
int32_t realearth_step(float delta_time, float throttle, float steer, float brake);

/**
 * Retrieves the current vehicle and planetary telemetry.
 * @param out_telem Pointer to destination telemetry buffer.
 * @return 0 on success, non-zero on error.
 */
int32_t realearth_get_telemetry(RealEarthTelemetry* out_telem);

/**
 * Pauses simulation execution.
 * @return 0 on success.
 */
int32_t realearth_pause(void);

/**
 * Resumes simulation execution.
 * @return 0 on success.
 */
int32_t realearth_resume(void);

/**
 * Applies camera orbit gesture rotation deltas in radians.
 */
int32_t realearth_touch_orbit(float delta_yaw, float delta_pitch);

/**
 * Applies camera pinch-to-zoom distance delta in meters.
 */
int32_t realearth_touch_zoom(float delta_dist);

/**
 * Sets touch driving controls.
 */
int32_t realearth_set_touch_inputs(float steer, float throttle, float brake, int32_t handbrake);

/**
 * Evaluates surveyed ground elevation at local Cartesian coordinate (x, z).
 */
float realearth_get_ground_elevation(float x, float z);

/**
 * Attaches native surface window handle (ANativeWindow or MTKView).
 */
int32_t realearth_set_surface(uintptr_t surface_handle, int32_t width, int32_t height);

/**
 * Updates surface viewport dimensions.
 */
int32_t realearth_resize_surface(int32_t width, int32_t height);

/**
 * Detaches native surface window handle.
 */
int32_t realearth_destroy_surface(void);

/**
 * Mobile runtime status enum values:
 * 0 = Active GPU swapchain (ANativeWindow / CAMetalLayer)
 * 1 = Blocked pending native swapchain (host toolchain required)
 * 2 = Uninitialized
 */
int32_t realearth_mobile_runtime_status(void);

/**
 * Returns human-readable status disclosure string.
 */
const char* realearth_mobile_status_string(void);

/**
 * Ticks one render pass on the attached surface.
 * Notice: Presentation requires linking ANativeWindow/CAMetalLayer.
 * On desktop/host builds, this diagnostic stub increments frame count for backwards compatibility.
 */
int32_t realearth_render_frame(void);

/**
 * Evicts quiescent caches under low-memory OS pressure.
 */
int32_t realearth_trim_memory(int32_t level);

/**
 * Clamps target frame rate and interval to manage device thermals.
 */
int32_t realearth_set_thermal_throttle(float target_fps);

/**
 * Shuts down the engine and releases allocated resources.
 * @return 0 on success, non-zero on error.
 */
int32_t realearth_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif /* REALEARTH_H */
