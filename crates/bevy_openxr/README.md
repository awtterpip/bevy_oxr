# Bevy mod OpenXR

This is the OpenXR backend for bevy_mod_xr for more info see the [repo](https://github.com/awtterpip/bevy_oxr)

## Stereo multiview

The default stereo path renders each eye through its own 2D texture view. This is the most compatible mode and is what Bevy's built-in 3D render graph expects today.

Apps that provide their own multiview-aware render graph nodes can ask the OpenXR backend to also expose a 2-layer array texture view for the current stereo swapchain image:

```rust
use bevy::prelude::*;
use bevy_mod_openxr::add_xr_plugins;
use bevy_mod_xr::camera::XrStereoRenderMode;

fn main() {
    App::new()
        .insert_resource(XrStereoRenderMode::stereo_multiview())
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .run();
}
```

The multiview texture view is available from `bevy_mod_openxr::render::xr_multiview_texture_view_handle()`. Custom render passes should use that manual texture view and set `wgpu::RenderPassDescriptor::multiview_mask` to `XrStereoRenderMode::view_mask()`.

The render device must support `wgpu::Features::MULTIVIEW` before a custom pass uses a non-empty multiview mask.

This does not automatically convert Bevy's built-in PBR or core render passes to single-pass stereo. Those passes still need multiview-aware render graph integration and shaders that select per-eye data with the platform view index.

See `cargo run -p bevy_mod_openxr --example multiview` for a minimal setup example.

![](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExY2FlOXJrOG1pbzFkYTVjZHIybndqamF1a2YwZHU3dXgyZGcwdmFzMiZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/CHbQyXOT5yZZ1VQRh7/giphy-downsized-large.gif)
![](https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExbHVmZXc2b3VhcGE2eHE2c2Y3NDR6cXNibHdjNjk5MmtyOHlkMXkwZyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/Hsvp5el2o7tzgOf9GQ/giphy-downsized-large.gif)
