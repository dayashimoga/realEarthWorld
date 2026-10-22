#!/usr/bin/env bash
# Real Earth Platform — Build Android Native JNI Shared Library (librealearth.so)
set -euo pipefail

echo "=========================================================="
echo "   BUILDING REAL EARTH ANDROID JNI (aarch64-linux-android)"
echo "=========================================================="

# Check if cargo-ndk is installed
if command -v cargo-ndk &> /dev/null; then
    echo "[INFO] Using cargo-ndk toolchain wrapper..."
    cargo ndk --target aarch64-linux-android --platform 31 build -p realearth-mobile --release
elif [ -n "${ANDROID_NDK_HOME:-}" ]; then
    echo "[INFO] ANDROID_NDK_HOME detected at: $ANDROID_NDK_HOME"
    NDK_LLVM_BIN=$(echo "$ANDROID_NDK_HOME"/toolchains/llvm/prebuilt/*/bin)
    
    if [ ! -d "$NDK_LLVM_BIN" ]; then
        echo "[ERROR] NDK LLVM toolchain directory not found: $NDK_LLVM_BIN"
        exit 1
    fi
    
    export PATH="$NDK_LLVM_BIN:$PATH"
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_LLVM_BIN/aarch64-linux-android31-clang"
    export CC_aarch64_linux_android="$NDK_LLVM_BIN/aarch64-linux-android31-clang"
    export AR_aarch64_linux_android="$NDK_LLVM_BIN/llvm-ar"
    
    echo "[INFO] Configured cross-linker: $CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"
    cargo build -p realearth-mobile --target aarch64-linux-android --release
else
    echo "[ERROR] Neither 'cargo-ndk' nor 'ANDROID_NDK_HOME' was found."
    echo "[HINT] Install cargo-ndk via 'cargo install cargo-ndk' or export ANDROID_NDK_HOME."
    exit 1
fi

OUTPUT_SO="target/aarch64-linux-android/release/librealearth.so"
if [ -f "$OUTPUT_SO" ]; then
    echo "[SUCCESS] Android JNI shared library built successfully at: $OUTPUT_SO"
    file "$OUTPUT_SO" || true
else
    echo "[ERROR] Expected build output $OUTPUT_SO not found."
    exit 1
fi
