#!/bin/bash
set -e
cargo build --target=arm-linux-androideabi --release
cd ./android
./gradlew appStart 
adb logcat