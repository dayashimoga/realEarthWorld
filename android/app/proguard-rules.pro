# Real Earth Android ProGuard / R8 Rules
-keep class com.realearth.world.RealEarthEngine { *; }
-keepclasseswithmembernames class * {
    native <methods>;
}
