#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SDK="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-$HOME/Android/Sdk}}"
if [ ! -d "$SDK" ] && [ -d "/home/ubuntu/android-sdk" ]; then
    SDK="/home/ubuntu/android-sdk"
fi
NDK="${ANDROID_NDK_HOME:-$SDK/ndk/27.2.12479018}"
GRADLE="${GRADLE_BIN:-$(command -v gradle || true)}"
if [ -z "$GRADLE" ] || [ ! -x "$GRADLE" ]; then
    echo "Gradle was not found. Install a compatible Gradle version or set GRADLE_BIN." >&2
    exit 1
fi

: "${ANDROID_SDK_ROOT:=$SDK}"
: "${ANDROID_NDK_HOME:=$NDK}"
export ANDROID_SDK_ROOT ANDROID_NDK_HOME

. "$HOME/.cargo/env"
cd "$ROOT"
mkdir -p "$ROOT/packaging/android/app/src/main/jniLibs"
cargo ndk -t arm64-v8a -t x86_64 -o "$ROOT/packaging/android/app/src/main/jniLibs" build -p apelles-android --release
printf 'sdk.dir=%s\n' "$ANDROID_SDK_ROOT" > "$ROOT/packaging/android/local.properties"
cd "$ROOT/packaging/android"
"$GRADLE" --no-daemon assembleRelease
printf '\nAPK: %s\n' "$ROOT/packaging/android/app/build/outputs/apk/release/app-release-unsigned.apk"
