#import "RealEarthBridge.h"

@implementation RealEarthBridge

+ (BOOL)initializeWithRegion:(NSString *)region {
    const char *c_region = [region UTF8String];
    return realearth_init(c_region) == 0;
}

+ (BOOL)stepSimulationWithDeltaTime:(float)dt
                           throttle:(float)throttle
                              steer:(float)steer
                              brake:(float)brake {
    return realearth_step(dt, throttle, steer, brake) == 0;
}

+ (nullable NSDictionary *)currentTelemetry {
    RealEarthTelemetry telem;
    if (realearth_get_telemetry(&telem) != 0 || telem.is_active == 0) {
        return nil;
    }
    return @{
        @"lat": @(telem.lat),
        @"lon": @(telem.lon),
        @"alt": @(telem.alt),
        @"speed_kph": @(telem.speed_kph),
        @"engine_rpm": @(telem.engine_rpm),
        @"gear": @(telem.gear),
        @"suspension_fl": @(telem.suspension_fl),
        @"suspension_fr": @(telem.suspension_fr),
    };
}

+ (void)pause {
    realearth_pause();
}

+ (void)resume {
    realearth_resume();
}

+ (void)renderFrame {
    realearth_render_frame();
}

+ (void)trimMemory:(int)level {
    realearth_trim_memory(level);
}

+ (void)touchOrbitWithDeltaYaw:(float)deltaYaw deltaPitch:(float)deltaPitch {
    realearth_touch_orbit(deltaYaw, deltaPitch);
}

+ (void)touchZoomWithDeltaDistance:(float)deltaDist {
    realearth_touch_zoom(deltaDist);
}

+ (float)groundElevationAtX:(float)x z:(float)z {
    return realearth_get_ground_elevation(x, z);
}

+ (void)shutdown {
    realearth_shutdown();
}

@end
