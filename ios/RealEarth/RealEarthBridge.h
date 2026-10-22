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
+ (void)shutdown;

@end

NS_ASSUME_NONNULL_END
