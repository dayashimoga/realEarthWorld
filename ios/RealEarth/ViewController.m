#import "ViewController.h"
#import "RealEarthBridge.h"

@interface ViewController () {
    float _throttle;
    float _steer;
    float _brake;
    CFTimeInterval _lastTimestamp;
}

@property (nonatomic, strong) UILabel *telemetryLabel;
@property (nonatomic, strong) UILabel *statusLabel;
@property (nonatomic, strong) UISegmentedControl *regionControl;
@property (nonatomic, strong) UISlider *steerSlider;
@property (nonatomic, strong) UIButton *throttleButton;
@property (nonatomic, strong) UIButton *brakeButton;

@end

@implementation ViewController

- (void)viewDidLoad {
    super.viewDidLoad();
    self.view.backgroundColor = [UIColor colorWithRed:0.06 green:0.09 blue:0.16 alpha:1.0];

    [self setupMetalView];
    [self setupHudUI];
    [self setupControls];

    // Initialize Rust engine with San Francisco pilot
    [RealEarthBridge initializeWithRegion:@"san-francisco"];

    // Start 60Hz display link
    self.displayLink = [CADisplayLink displayLinkWithTarget:self selector:@selector(renderLoop:)];
    [self.displayLink addToRunLoop:[NSRunLoop mainRunLoop] forMode:NSRunLoopCommonModes];
}

- (void)setupMetalView {
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    self.metalView = [[MTKView alloc] initWithFrame:self.view.bounds device:device];
    self.metalView.autoresizingMask = UIViewAutoresizingFlexibleWidth | UIViewAutoresizingFlexibleHeight;
    self.metalView.delegate = self;
    self.metalView.colorPixelFormat = MTLPixelFormatBGRA8Unorm;
    self.metalView.clearColor = MTLClearColorMake(0.06, 0.09, 0.16, 1.0);
    [self.view addSubview:self.metalView];
}

- (void)setupHudUI {
    // Top Region Selector
    NSArray *regions = @[@"SF", @"NYC", @"LA", @"CHI", @"SEA", @"MIA", @"BLR"];
    self.regionControl = [[UISegmentedControl alloc] initWithItems:regions];
    self.regionControl.selectedSegmentIndex = 0;
    self.regionControl.translatesAutoresizingMaskIntoConstraints = NO;
    [self.regionControl addTarget:self action:@selector(regionChanged:) forControlEvents:UIControlEventValueChanged];
    [self.view addSubview:self.regionControl];

    // Status Label
    self.statusLabel = [[UILabel alloc] init];
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.statusLabel.textColor = [UIColor colorWithRed:0.22 green:0.74 blue:0.97 alpha:1.0];
    self.statusLabel.font = [UIFont boldSystemFontOfSize:14];
    self.statusLabel.text = @"Real Earth iOS • Metal 60 FPS";
    [self.view addSubview:self.statusLabel];

    // Telemetry HUD Card
    self.telemetryLabel = [[UILabel alloc] init];
    self.telemetryLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.telemetryLabel.numberOfLines = 0;
    self.telemetryLabel.backgroundColor = [UIColor colorWithWhite:0.0 alpha:0.65];
    self.telemetryLabel.layer.cornerRadius = 8.0;
    self.telemetryLabel.layer.masksToBounds = YES;
    self.telemetryLabel.textColor = [UIColor whiteColor];
    self.telemetryLabel.font = [UIFont monospacedDigitSystemFontOfSize:13 weight:UIFontWeightMedium];
    self.telemetryLabel.text = @" Speed: 0.0 km/h\n Gear: D (1)\n RPM: 800\n GPS: 37.7749 N, -122.4194 W";
    [self.view addSubview:self.telemetryLabel];

    [NSLayoutConstraint activateConstraints:@[
        [self.statusLabel.topAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.topAnchor constant:8],
        [self.statusLabel.leadingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.leadingAnchor constant:16],

        [self.regionControl.topAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.topAnchor constant:8],
        [self.regionControl.trailingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.trailingAnchor constant:-16],

        [self.telemetryLabel.topAnchor constraintEqualToAnchor:self.statusLabel.bottomAnchor constant:12],
        [self.telemetryLabel.leadingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.leadingAnchor constant:16],
        [self.telemetryLabel.widthAnchor constraintEqualToConstant:240],
        [self.telemetryLabel.heightAnchor constraintEqualToConstant:85]
    ]];
}

- (void)setupControls {
    // Steering Slider
    self.steerSlider = [[UISlider alloc] init];
    self.steerSlider.translatesAutoresizingMaskIntoConstraints = NO;
    self.steerSlider.minimumValue = -1.0;
    self.steerSlider.maximumValue = 1.0;
    self.steerSlider.value = 0.0;
    [self.steerSlider addTarget:self action:@selector(steerChanged:) forControlEvents:UIControlEventValueChanged];
    [self.steerSlider addTarget:self action:@selector(steerReleased:) forControlEvents:UIControlEventTouchUpInside | UIControlEventTouchUpOutside];
    [self.view addSubview:self.steerSlider];

    // Brake Button
    self.brakeButton = [UIButton buttonWithType:UIButtonTypeSystem];
    self.brakeButton.translatesAutoresizingMaskIntoConstraints = NO;
    [self.brakeButton setTitle:@"BRAKE" forState:UIControlStateNormal];
    [self.brakeButton setTitleColor:[UIColor whiteColor] forState:UIControlStateNormal];
    self.brakeButton.titleLabel.font = [UIFont boldSystemFontOfSize:14];
    self.brakeButton.backgroundColor = [UIColor colorWithRed:0.94 green:0.27 blue:0.27 alpha:1.0];
    self.brakeButton.layer.cornerRadius = 8.0;
    [self.brakeButton addTarget:self action:@selector(brakeDown) forControlEvents:UIControlEventTouchDown];
    [self.brakeButton addTarget:self action:@selector(brakeUp) forControlEvents:UIControlEventTouchUpInside | UIControlEventTouchUpOutside | UIControlEventTouchCancel];
    [self.view addSubview:self.brakeButton];

    // Throttle Button
    self.throttleButton = [UIButton buttonWithType:UIButtonTypeSystem];
    self.throttleButton.translatesAutoresizingMaskIntoConstraints = NO;
    [self.throttleButton setTitle:@"GAS" forState:UIControlStateNormal];
    [self.throttleButton setTitleColor:[UIColor whiteColor] forState:UIControlStateNormal];
    self.throttleButton.titleLabel.font = [UIFont boldSystemFontOfSize:14];
    self.throttleButton.backgroundColor = [UIColor colorWithRed:0.13 green:0.77 blue:0.37 alpha:1.0];
    self.throttleButton.layer.cornerRadius = 8.0;
    [self.throttleButton addTarget:self action:@selector(throttleDown) forControlEvents:UIControlEventTouchDown];
    [self.throttleButton addTarget:self action:@selector(throttleUp) forControlEvents:UIControlEventTouchUpInside | UIControlEventTouchUpOutside | UIControlEventTouchCancel];
    [self.view addSubview:self.throttleButton];

    [NSLayoutConstraint activateConstraints:@[
        [self.steerSlider.bottomAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.bottomAnchor constant:-20],
        [self.steerSlider.leadingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.leadingAnchor constant:24],
        [self.steerSlider.widthAnchor constraintEqualToConstant:220],

        [self.brakeButton.bottomAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.bottomAnchor constant:-16],
        [self.brakeButton.trailingAnchor constraintEqualToAnchor:self.throttleButton.leadingAnchor constant:-16],
        [self.brakeButton.widthAnchor constraintEqualToConstant:80],
        [self.brakeButton.heightAnchor constraintEqualToConstant:60],

        [self.throttleButton.bottomAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.bottomAnchor constant:-16],
        [self.throttleButton.trailingAnchor constraintEqualToAnchor:self.view.safeAreaLayoutGuide.trailingAnchor constant:-24],
        [self.throttleButton.widthAnchor constraintEqualToConstant:80],
        [self.throttleButton.heightAnchor constraintEqualToConstant:60]
    ]];
}

- (void)regionChanged:(UISegmentedControl *)sender {
    NSArray *regionKeys = @[@"san-francisco", @"new-york", @"los-angeles", @"chicago", @"seattle", @"miami", @"bengaluru"];
    NSString *selected = regionKeys[sender.selectedSegmentIndex];
    [RealEarthBridge initializeWithRegion:selected];
}

- (void)steerChanged:(UISlider *)sender {
    _steer = sender.value;
}

- (void)steerReleased:(UISlider *)sender {
    sender.value = 0.0;
    _steer = 0.0;
}

- (void)throttleDown { _throttle = 1.0f; }
- (void)throttleUp { _throttle = 0.0f; }
- (void)brakeDown { _brake = 1.0f; }
- (void)brakeUp { _brake = 0.0f; }

- (void)renderLoop:(CADisplayLink *)link {
    if (_lastTimestamp == 0) {
        _lastTimestamp = link.timestamp;
        return;
    }
    float dt = (float)(link.timestamp - _lastTimestamp);
    _lastTimestamp = link.timestamp;

    // Step physics & vehicle simulation in Rust engine
    [RealEarthBridge stepSimulationWithDeltaTime:dt throttle:_throttle steer:_steer brake:_brake];

    // Readback telemetry
    NSDictionary *telem = [RealEarthBridge currentTelemetry];
    if (telem) {
        self.telemetryLabel.text = [NSString stringWithFormat:
            @" Speed: %.1f km/h\n Gear: %d\n RPM: %.0f\n GPS: %.4f N, %.4f W",
            [telem[@"speed_kph"] floatValue],
            [telem[@"gear"] intValue],
            [telem[@"engine_rpm"] floatValue],
            [telem[@"lat"] doubleValue],
            [telem[@"lon"] doubleValue]
        ];
    }
}

- (void)drawInMTKView:(MTKView *)view {
    // Metal frame rendering pass
}

- (void)mtkView:(MTKView *)view drawableSizeWillChange:(CGSize)size {
    // Handle view resize or orientation rotation
}

- (void)didReceiveMemoryWarning {
    [super didReceiveMemoryWarning];
    NSLog(@"[RealEarth iOS] Memory Warning received — evicting chunk streaming cache.");
}

- (void)dealloc {
    [self.displayLink invalidate];
    [RealEarthBridge shutdown];
}

@end
