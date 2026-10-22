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
 * Shuts down the engine and releases allocated resources.
 * @return 0 on success, non-zero on error.
 */
int32_t realearth_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif /* REALEARTH_H */
