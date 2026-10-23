package com.realearth.world;

import android.app.Activity;
import android.content.Context;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.PowerManager;
import android.util.Log;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;
import android.view.View;
import android.widget.AdapterView;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.SeekBar;
import android.widget.Spinner;
import android.widget.TextView;

import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Real Earth Android Main Activity.
 *
 * Provides native SurfaceView rendering lifecycle, virtual driving controls,
 * real-time telemetry HUD, region selection, and thermal/memory management.
 */
public class MainActivity extends Activity implements SurfaceHolder.Callback {
    private static final String TAG = "RealEarthApp";

    private SurfaceView surfaceView;
    private TextView tvSpeed;
    private TextView tvGear;
    private TextView tvRpm;
    private TextView tvCoordinates;
    private TextView tvThermal;
    private Spinner spinnerRegion;
    private SeekBar seekSteer;
    private Button btnThrottle;
    private Button btnBrake;

    private final Handler mainHandler = new Handler(Looper.getMainLooper());
    private Thread simThread;
    private final AtomicBoolean isRunning = new AtomicBoolean(false);

    // Vehicle control state
    private volatile float currentThrottle = 0.0f;
    private volatile float currentSteer = 0.0f;
    private volatile float currentBrake = 0.0f;

    // Thermal throttling state (1.0 = full 60fps, 0.5 = 30fps throttle)
    private volatile float targetFrameIntervalMs = 16.67f;

    private static final String[] SUPPORTED_REGIONS = {
        "San Francisco (SF)",
        "New York City (NYC)",
        "Los Angeles (LA)",
        "Chicago (CHI)",
        "Seattle (SEA)",
        "Miami (MIA)",
        "Bengaluru (BLR)"
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        surfaceView = findViewById(R.id.surface_view);
        tvSpeed = findViewById(R.id.tv_speed);
        tvGear = findViewById(R.id.tv_gear);
        tvRpm = findViewById(R.id.tv_rpm);
        tvCoordinates = findViewById(R.id.tv_coordinates);
        tvThermal = findViewById(R.id.tv_thermal);
        spinnerRegion = findViewById(R.id.spinner_region);
        seekSteer = findViewById(R.id.seek_steer);
        btnThrottle = findViewById(R.id.btn_throttle);
        btnBrake = findViewById(R.id.btn_brake);

        surfaceView.getHolder().addCallback(this);

        setupRegionSpinner();
        setupDrivingControls();
        setupThermalMonitoring();

        // Initialize Rust simulation engine with default region
        int initResult = RealEarthEngine.nativeInit("san-francisco");
        Log.i(TAG, "Real Earth Engine initialized: " + initResult);
    }

    private void setupRegionSpinner() {
        ArrayAdapter<String> adapter = new ArrayAdapter<>(
            this,
            android.R.layout.simple_spinner_item,
            SUPPORTED_REGIONS
        );
        adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spinnerRegion.setAdapter(adapter);

        spinnerRegion.setOnItemSelectedListener(new AdapterView.OnItemSelectedListener() {
            @Override
            public void onItemSelected(AdapterView<?> parent, View view, int position, long id) {
                String selected = SUPPORTED_REGIONS[position];
                String regionKey = selected.toLowerCase();
                Log.i(TAG, "Switching active region to: " + regionKey);
                RealEarthEngine.nativeInit(regionKey);
            }

            @Override
            public void onNothingSelected(AdapterView<?> parent) {}
        });
    }

    private void setupDrivingControls() {
        // Steering slider (-1.0 to +1.0)
        seekSteer.setMax(200);
        seekSteer.setProgress(100);
        seekSteer.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            @Override
            public void onProgressChanged(SeekBar seekBar, int progress, boolean fromUser) {
                currentSteer = (progress - 100) / 100.0f;
            }

            @Override
            public void onStartTrackingTouch(SeekBar seekBar) {}

            @Override
            public void onStopTrackingTouch(SeekBar seekBar) {
                // Auto-center steering when released
                seekBar.setProgress(100);
                currentSteer = 0.0f;
            }
        });

        // Touch throttle pedal
        btnThrottle.setOnTouchListener((v, event) -> {
            switch (event.getAction()) {
                case MotionEvent.ACTION_DOWN:
                    currentThrottle = 1.0f;
                    v.setPressed(true);
                    return true;
                case MotionEvent.ACTION_UP:
                case MotionEvent.ACTION_CANCEL:
                    currentThrottle = 0.0f;
                    v.setPressed(false);
                    return true;
            }
            return false;
        });

        // Touch brake pedal
        btnBrake.setOnTouchListener((v, event) -> {
            switch (event.getAction()) {
                case MotionEvent.ACTION_DOWN:
                    currentBrake = 1.0f;
                    v.setPressed(true);
                    return true;
                case MotionEvent.ACTION_UP:
                case MotionEvent.ACTION_CANCEL:
                    currentBrake = 0.0f;
                    v.setPressed(false);
                    return true;
            }
            return false;
        });
    }

    private void setupThermalMonitoring() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            PowerManager powerManager = (PowerManager) getSystemService(Context.POWER_SERVICE);
            if (powerManager != null) {
                powerManager.addThermalStatusListener(status -> {
                    switch (status) {
                        case PowerManager.THERMAL_STATUS_NONE:
                        case PowerManager.THERMAL_STATUS_LIGHT:
                            targetFrameIntervalMs = 16.67f; // 60 FPS
                            tvThermal.setText("Thermal: Nominal (60 FPS)");
                            break;
                        case PowerManager.THERMAL_STATUS_MODERATE:
                            targetFrameIntervalMs = 22.22f; // 45 FPS
                            tvThermal.setText("Thermal: Moderate (45 FPS)");
                            break;
                        case PowerManager.THERMAL_STATUS_SEVERE:
                        case PowerManager.THERMAL_STATUS_CRITICAL:
                            targetFrameIntervalMs = 33.33f; // 30 FPS throttle
                            tvThermal.setText("Thermal: Severe (30 FPS Throttled)");
                            break;
                        case PowerManager.THERMAL_STATUS_EMERGENCY:
                        case PowerManager.THERMAL_STATUS_SHUTDOWN:
                            targetFrameIntervalMs = 50.00f; // 20 FPS extreme throttle
                            tvThermal.setText("Thermal: Emergency (20 FPS)");
                            break;
                    }
                });
            }
        } else {
            tvThermal.setText("Thermal: Standard (60 FPS)");
        }
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        Log.i(TAG, "Native Surface created, linking to WGPU rendering.");
        RealEarthEngine.nativeSurfaceCreated(holder.getSurface());
        startSimulationLoop();
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        Log.i(TAG, "Surface resized: " + width + "x" + height);
        RealEarthEngine.nativeSurfaceChanged(holder.getSurface(), width, height);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        Log.i(TAG, "Surface destroyed, detaching WGPU surface.");
        RealEarthEngine.nativeSurfaceDestroyed();
        stopSimulationLoop();
    }

    private void startSimulationLoop() {
        isRunning.set(true);
        simThread = new Thread(() -> {
            long lastTick = System.nanoTime();

            while (isRunning.get()) {
                long now = System.nanoTime();
                float dt = (now - lastTick) / 1_000_000_000.0f;
                lastTick = now;

                // Step Rust native engine
                RealEarthEngine.nativeStep(dt, currentThrottle, currentSteer, currentBrake);

                // Update UI telemetry on main thread periodically
                mainHandler.post(this::updateTelemetryDisplay);

                // Frame rate regulation
                try {
                    Thread.sleep((long) targetFrameIntervalMs);
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
        }, "RealEarthSimThread");
        simThread.start();
    }

    private void stopSimulationLoop() {
        isRunning.set(false);
        if (simThread != null) {
            try {
                simThread.join(1000);
            } catch (InterruptedException ignored) {}
            simThread = null;
        }
    }

    private void updateTelemetryDisplay() {
        float speed = RealEarthEngine.nativeGetSpeed();
        double lat = RealEarthEngine.nativeGetLat();
        double lon = RealEarthEngine.nativeGetLon();
        double alt = RealEarthEngine.nativeGetAlt();
        tvSpeed.setText(String.format("Speed: %.1f km/h", speed));
        tvCoordinates.setText(String.format("%.4f°N, %.4f°W (Alt: %.1fm)", lat, lon, alt));
    }

    @Override
    protected void onPause() {
        super.onPause();
        stopSimulationLoop();
    }

    @Override
    protected void onResume() {
        super.onResume();
        if (surfaceView.getHolder().getSurface().isValid()) {
            startSimulationLoop();
        }
    }

    @Override
    public void onTrimMemory(int level) {
        super.onTrimMemory(level);
        Log.w(TAG, "TrimMemory received level: " + level + " — triggering cache prune.");
        RealEarthEngine.nativeTrimMemory(level);
    }

    @Override
    public void onLowMemory() {
        super.onLowMemory();
        Log.w(TAG, "LowMemory event received — flushing chunk caches.");
        RealEarthEngine.nativeTrimMemory(80);
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        stopSimulationLoop();
        RealEarthEngine.nativeShutdown();
        Log.i(TAG, "Real Earth Engine shutdown complete.");
    }
}
