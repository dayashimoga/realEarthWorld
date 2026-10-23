package com.realearth.world;

import android.view.Surface;

/**
 * Android JNI Bridge to Real Earth Rust Native Engine (librealearth.so).
 *
 * Implements genuine JNI ABI signatures for region initialization, physics/telemetry stepping,
 * native SurfaceView presentation, thermal regulation, memory trimming, touch controls,
 * camera orbit/zoom gestures, and DEM ground queries.
 */
public final class RealEarthEngine {
    static {
        System.loadLibrary("realearth");
    }

    // Lifecycle and simulation stepping
    public static native int nativeInit(String region);
    public static native int nativeStep(float dt, float throttle, float steer, float brake);
    public static native int nativeShutdown();
    public static native int nativePause();
    public static native int nativeResume();

    // Planetary and vehicle telemetry queries
    public static native float nativeGetSpeed();
    public static native double nativeGetLat();
    public static native double nativeGetLon();
    public static native double nativeGetAlt();
    public static native float nativeGetGroundElevation(float x, float z);

    // Native Surface & WGPU presentation lifecycle
    public static native int nativeSurfaceCreated(Surface surface);
    public static native int nativeSurfaceChanged(Surface surface, int width, int height);
    public static native int nativeSurfaceDestroyed();
    public static native int nativeRenderFrame();

    // Mobile hardware adaptation
    public static native int nativeSetThermalThrottle(float targetFps);
    public static native int nativeTrimMemory(int level);

    // Touch and gesture controls
    public static native int nativeTouchOrbit(float deltaYaw, float deltaPitch);
    public static native int nativeTouchZoom(float deltaDist);

    private RealEarthEngine() {}
}
