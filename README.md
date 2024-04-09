# Bevy OpenXR

A crate for adding openxr support to Bevy ( planned to be upstreamed ). 

To see it in action run the example in `examples` with `cargo run --example xr`

## Discord
Come hang out if you have questions or issues 
https://discord.gg/sqMw7UJhNc

![](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExY2FlOXJrOG1pbzFkYTVjZHIybndqamF1a2YwZHU3dXgyZGcwdmFzMiZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/CHbQyXOT5yZZ1VQRh7/giphy-downsized-large.gif)
![](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExbHVmZXc2b3VhcGE2eHE2c2Y3NDR6cXNibHdjNjk5MmtyOHlkMXkwZyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/Hsvp5el2o7tzgOf9GQ/giphy-downsized-large.gif)
## Troubleshooting

- Make sure, if you're on Linux, that you have the `openxr` package installed on your system.
- I'm getting poor performance.
    - Like other bevy projects, make sure you're building in release (example: `cargo run --example xr --release`)
