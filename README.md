# rust-webvr
Safe rust API that provides a way to interact with Virtual Reality headsets and integration with vendor specific SDKs like OpenVR, Oculus and GoogleVR (Daydream). The API is inspired on the easy to use WebVR API but adapted to Rust design patterns.

It's used in the WebVR Core implementation for Servo browser. This module can be tested outside of Servo and even be used on any vanilla Rust app.

## Room Scale example: 

Just run this command in examples/room folder

```
cargo run
```

Run room scale demo on android:

```
./run_android.sh
```

### OpenVR tips:

In order to run with openvr on windows, `openvr_api.dll` must be available. Please make it either accessible in your path, or copy it into the examples/room folder.

Refer to [The ValveSoftware openvr repository](https://github.com/ValveSoftware/openvr) and head over to the releases section for more information.
