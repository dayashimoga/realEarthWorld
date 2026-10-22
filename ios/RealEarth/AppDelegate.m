#import "AppDelegate.h"
#import "ViewController.h"
#import "RealEarthBridge.h"

@implementation AppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    self.window = [[UIWindow alloc] initWithFrame:[[UIScreen mainScreen] bounds]];
    ViewController *viewController = [[ViewController alloc] init];
    self.window.rootViewController = viewController;
    [self.window makeKeyAndVisible];
    return YES;
}

- (void)applicationWillResignActive:(UIApplication *)application {
    // Pause rendering and vehicle simulation loop when resigning active
    NSLog(@"[RealEarth iOS] Application will resign active.");
}

- (void)applicationDidEnterBackground:(UIApplication *)application {
    // Release transient memory and non-essential caches
    NSLog(@"[RealEarth iOS] Application entered background.");
}

- (void)applicationWillEnterForeground:(UIApplication *)application {
    // Re-verify Metal pipeline and resume simulation loop
    NSLog(@"[RealEarth iOS] Application will enter foreground.");
}

- (void)applicationDidBecomeActive:(UIApplication *)application {
    NSLog(@"[RealEarth iOS] Application became active.");
}

- (void)applicationWillTerminate:(UIApplication *)application {
    [RealEarthBridge shutdown];
    NSLog(@"[RealEarth iOS] Application terminated, engine shutdown.");
}

@end
