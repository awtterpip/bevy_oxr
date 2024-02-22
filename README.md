# Bevy OpenXR

A crate for adding openxr support to Bevy ( planned to be upstreamed ). 
![image](https://github.com/awtterpip/bevy_openxr/assets/50841145/aa01fde4-7915-49b9-b486-ff61ce6d57a9)

To see it in action run the example in `examples` with `cargo run --example xr`

## Discord
Come hang out if you have questions or issues 
https://discord.gg/M5UTBsjN

## Troubleshooting

- Make sure, if you're on Linux, that you have the `openxr` package installed on your system.
- I'm getting poor performance.
    - Like other bevy projects, make sure you're building in release (example: `cargo run --example xr --release`)

## Recommendations

for now we recommend you to add the folowing to your root Cargo.toml
```
[patch.crates-io]
ndk = { git = "https://github.com/Schmarni-Dev/ndk.git", branch = "070" }
ndk-sys = { package = "ndk-sys", git = "https://github.com/Schmarni-Dev/ndk.git", branch = "070" }
ndk-context = { package = "ndk-context", git = "https://github.com/Schmarni-Dev/ndk.git", branch = "070" }
bevy_pbr = { package = "bevy_pbr", git = "https://github.com/MalekiRe/bevy", branch = "release-0.12.1" }
```
