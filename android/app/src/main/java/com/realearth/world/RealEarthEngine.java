package com.realearth.world;

import android.view.Surface;

/**
 * Android JNI Bridge to Real Earth Rust Native Engine (librealearth.so).
 *
 * Implements genuine JNI ABI signatures for region initialization, physics/telemetry stepping,
 * native SurfaceView presentation, thermal regulation, and memory trimming.
 */
public final class RealEarthEngine {
    static {
        System.loadLibrary("realearth");
    }

    // Lifecycle and simulation stepping
    public static native int nativeInit(String region);
    public static native int nativeStep(float dt, float throttle, float steer, float brake);
    public static native int nativeShutdown();

    // Planetary and vehicle telemetry queries
    public static native float nativeGetSpeed();
    public static native double nativeGetLat();
    public static native double nativeGetLon();
    public static native double nativeGetAlt();

    // Native Surface & WGPU presentation lifecycle
    public static native int nativeSurfaceCreated(Surface surface);
    public static native int nativeSurfaceChanged(Surface surface, int width, int height);
    public static native int nativeSurfaceDestroyed();

    // Mobile hardware adaptation
    public static native int nativeSetThermalThrottle(float targetFps);
    public static native int nativeTrimMemory(int level);

    private RealEarthEngine() {}
}
