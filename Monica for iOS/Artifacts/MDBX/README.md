# MDBX Binary Artifacts

`Scripts/build-mdbx-xcframework.sh` writes `MonicaMDBXGenerated.xcframework` here.

The XCFramework is ignored because it is a local build artifact. Rebuild it from the Rust source and generated UniFFI headers when setting up the iOS app or CI.
