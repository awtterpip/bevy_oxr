# Bevy OpenXR

An in-progress crate for adding openxr support to Bevy without forking. 
![image](https://github.com/awtterpip/bevy_openxr/assets/50841145/aa01fde4-7915-49b9-b486-ff61ce6d57a9)

To see it in action run the example in `examples` with `cargo run --example xr`

## Troubleshooting

- Make sure, if you're on Linux, that you have the `openxr` package installed on your system.
- I'm getting poor performance.
    - Like other bevy projects, make sure you're building in release (example: `cargo run --example xr --release`)
