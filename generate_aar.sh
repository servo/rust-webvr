#!/bin/sh

set -e

cd ./rust-webvr/src/api/googlevr/gradle
./gradlew assembleRelease
cp ./build/outputs/aar/GVRService-release.aar ../aar/GVRService.aar
cd -

cd ./rust-webvr/src/api/oculusvr/gradle
./gradlew assembleRelease
cp ./build/outputs/aar/OVRService-release.aar ../aar/OVRService.aar
cd -

