# Bevy OpenXR

An in-progress crate for adding openxr support to Bevy without forking. 
![image](https://github.com/awtterpip/bevy_openxr/assets/50841145/aa01fde4-7915-49b9-b486-ff61ce6d57a9)

To see it in action run the example in `examples` with `cargo run --example xr`

## Troubleshooting

- I'm getting a `CMake error: ...` on Linux.
    - Make sure you have the `openxr` package installed on your system.
    - Append `--no-default-features` to your build command (example: `cargo run --example xr --no-default-features`)
- I'm getting poor performance.
    - Like other bevy projects, make sure you're building in release (example: `cargo run --example xr --release`)
