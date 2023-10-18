# Bevy OpenXR

An in-progress crate for adding openxr support to Bevy without forking. 
![image](https://github.com/awtterpip/bevy_openxr/assets/50841145/aa01fde4-7915-49b9-b486-ff61ce6d57a9)

To see it in action run the example in `examples` with `cargo run --example xr`.

## Quest
Running on Meta Quest can be done with https://github.com/rust-mobile/cargo-apk and requires disabling default features. 
```sh 
cargo apk run --example xr --release --no-default-features
```
