#!/bin/bash
set -e
cargo build --target=arm-linux-androideabi --release
cd ./android
#./gradlew installGearvrArmRelease
./gradlew installDaydreamArmRelease
adb shell am start -n com.rust.webvr/com.rust.webvr.MainActivity
#logcat-color | egrep '(RustAndroid|art|DEBUG)'
