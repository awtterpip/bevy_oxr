# Bevy OpenXR Android example

## Setup
Get libopenxr_loader.so from the Oculus OpenXR Mobile SDK and add it to `examples/android/runtime_libs/arm64-v8a`
https://developer.oculus.com/downloads/package/oculus-openxr-mobile-sdk/
`examples/android/runtime_libs/arm64-v8a/libopenxr_loader.so`

Also, install either `cargo-apk` (marked as deprecated):
```sh
cargo install cargo-apk
```
or, install `xbuild` as it supersedes `cargo-apk`. Note that the `--git` is 
very important here.
```sh
cargo install --git https://github.com/rust-mobile/xbuild
```

## Run
Running on Meta Quest can be done with https://github.com/rust-mobile/cargo-apk. 
```sh 
cargo apk run --release
```
But cargo-apk is deprecated in favour of xbuild https://github.com/rust-mobile/xbuild.
```sh 
# List devices and copy device string "adb:***"
x devices

# Run on this device
x run --release --device adb:***
```
There is [manifest.yaml](./manifest.yaml) example required by xbuild.
Interface for this manifest can be found as AndroidConfig struct in https://github.com/rust-mobile/xbuild/blob/master/xbuild/src/config.rs

## Notes

### Relase mode
More optimisations enabled in Cargo.toml for the release mode. 
This gives more performance but longer build time.
```toml
[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"
```

### Cargo apk
If you see error like `Error: String `` is not a PID`, try to install cargo apk with a fix in branch.
```sh
cargo install --git https://github.com/rust-mobile/cargo-apk --branch=adb-logcat-uid
```

### Temporary JNIEnv log
This message is logged every frame. It's not yet fixed.
```sh
I JniUtils-inl: Creating temporary JNIEnv. This is a heavy operation and should be infrequent. To optimize, use JNI AttachCurrentThread on calling threa
```

### Android keystore
Release mode requires keystore. See Cargo.toml `package.metadata.android.signing.release`.

When creating your own apps, make sure to generate your own keystore, rather than using our example one!
You can use `keytool` like so:
```sh
keytool -genkey -v -keystore my-release-key.keystore -keyalg RSA -keysize 2048 -validity 10000
```
For more information on key signing and why it's so important, check out this article:
https://developer.android.com/studio/publish/app-signing
