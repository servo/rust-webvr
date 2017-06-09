#!/bin/bash
set -e
cargo build --target=arm-linux-androideabi --release
cd ./android
./gradlew installGearvrArmRelease
adb shell am start -n com.rust.webvr/com.rust.webvr.MainActivity
#logcat-color | egrep '(RustAndroid|art|DEBUG)'
