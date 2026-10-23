#import <Foundation/Foundation.h>
#include "../../bindings/mobile/include/realearth.h"

NS_ASSUME_NONNULL_BEGIN

@interface RealEarthBridge : NSObject

+ (BOOL)initializeWithRegion:(NSString *)region;
+ (BOOL)stepSimulationWithDeltaTime:(float)dt
                           throttle:(float)throttle
                              steer:(float)steer
                              brake:(float)brake;
+ (nullable NSDictionary *)currentTelemetry;
+ (void)pause;
+ (void)resume;
+ (void)renderFrame;
+ (void)trimMemory:(int)level;
+ (void)touchOrbitWithDeltaYaw:(float)deltaYaw deltaPitch:(float)deltaPitch;
+ (void)touchZoomWithDeltaDistance:(float)deltaDist;
+ (float)groundElevationAtX:(float)x z:(float)z;
+ (void)shutdown;

@end

NS_ASSUME_NONNULL_END
