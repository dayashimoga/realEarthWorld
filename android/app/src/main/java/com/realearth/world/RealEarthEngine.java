package com.realearth.world;

/**
 * Android JNI Bridge to Real Earth Rust Native Engine (librealearth.so).
 */
public final class RealEarthEngine {
    static {
        System.loadLibrary("realearth");
    }

    public static native int nativeInit(String region);
    public static native int nativeStep(float dt, float throttle, float steer, float brake);
    public static native int nativeShutdown();

    private RealEarthEngine() {}
}
