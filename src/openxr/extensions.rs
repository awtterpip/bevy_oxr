use bevy::prelude::{Deref, DerefMut};
use openxr::ExtensionSet;

#[derive(Clone, Debug, Eq, PartialEq, Deref, DerefMut)]
pub struct XrExtensions(ExtensionSet);
impl XrExtensions {
    pub fn raw_mut(&mut self) -> &mut ExtensionSet {
        &mut self.0
    }
    pub fn raw(&self) -> &ExtensionSet {
        &self.0
    }
    pub fn enable_fb_passthrough(&mut self) -> &mut Self {
        self.0.fb_passthrough = true;
        self
    }
    pub fn disable_fb_passthrough(&mut self) -> &mut Self {
        self.0.fb_passthrough = false;
        self
    }
    pub fn enable_hand_tracking(&mut self) -> &mut Self {
        self.0.ext_hand_tracking = true;
        self
    }
    pub fn disable_hand_tracking(&mut self) -> &mut Self {
        self.0.ext_hand_tracking = false;
        self
    }
    /// returns true if all of the extensions enabled are also available in `available_exts`
    pub fn is_available(&self, available_exts: &XrExtensions) -> bool {
        self.clone() & available_exts.clone() == *self
    }
}
impl From<ExtensionSet> for XrExtensions {
    fn from(value: ExtensionSet) -> Self {
        Self(value)
    }
}
impl From<XrExtensions> for ExtensionSet {
    fn from(val: XrExtensions) -> Self {
        val.0
    }
}
impl Default for XrExtensions {
    fn default() -> Self {
        let mut exts = ExtensionSet::default();
        exts.ext_hand_tracking = true;
        Self(exts)
    }
}

macro_rules! bitor {
    (
        $exts:ty;
        $(
            $(
                #[$meta:meta]
            )*
            $ident:ident
        ),*
        $(,)?
    ) => {
        impl std::ops::BitOr for $exts {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                let mut out = ExtensionSet::default();
                $(
                    $(
                        #[$meta]
                    )*
                    {
                        out.$ident = self.0.$ident || rhs.0.$ident;
                    }

                )*
                out.other = self.0.other;
                for ext in rhs.0.other {
                    if !out.other.contains(&ext) {
                        out.other.push(ext);
                    }
                }
                Self(out)
            }
        }
    };
}

macro_rules! bitand {
    (
        $exts:ty;
        $(
            $(
                #[$meta:meta]
            )*
            $ident:ident
        ),*
        $(,)?
    ) => {
        impl std::ops::BitAnd for $exts {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                let mut out = ExtensionSet::default();
                $(
                    $(
                        #[$meta]
                    )*
                    {
                        out.$ident = self.0.$ident && rhs.0.$ident;
                    }

                )*
                for ext in self.0.other {
                    if rhs.0.other.contains(&ext) {
                        out.other.push(ext);
                    }
                }
                Self(out)
            }
        }
    };
}

macro_rules! impl_ext {
    (
        $(
            $macro:ident
        ),*

    ) => {
        $(
            $macro! {
                XrExtensions;
                almalence_digital_lens_control,
                epic_view_configuration_fov,
                ext_performance_settings,
                ext_thermal_query,
                ext_debug_utils,
                ext_eye_gaze_interaction,
                ext_view_configuration_depth_range,
                ext_conformance_automation,
                ext_hand_tracking,
                #[cfg(windows)]
                ext_win32_appcontainer_compatible,
                ext_dpad_binding,
                ext_hand_joints_motion_range,
                ext_samsung_odyssey_controller,
                ext_hp_mixed_reality_controller,
                ext_palm_pose,
                ext_uuid,
                extx_overlay,
                fb_composition_layer_image_layout,
                fb_composition_layer_alpha_blend,
                #[cfg(target_os = "android")]
                fb_android_surface_swapchain_create,
                fb_swapchain_update_state,
                fb_composition_layer_secure_content,
                fb_display_refresh_rate,
                fb_color_space,
                fb_hand_tracking_mesh,
                fb_hand_tracking_aim,
                fb_hand_tracking_capsules,
                fb_spatial_entity,
                fb_foveation,
                fb_foveation_configuration,
                fb_keyboard_tracking,
                fb_triangle_mesh,
                fb_passthrough,
                fb_render_model,
                fb_spatial_entity_query,
                fb_spatial_entity_storage,
                fb_foveation_vulkan,
                #[cfg(target_os = "android")]
                fb_swapchain_update_state_android_surface,
                fb_swapchain_update_state_opengl_es,
                fb_swapchain_update_state_vulkan,
                fb_space_warp,
                fb_scene,
                fb_spatial_entity_container,
                fb_passthrough_keyboard_hands,
                fb_composition_layer_settings,
                htc_vive_cosmos_controller_interaction,
                htc_facial_tracking,
                htc_vive_focus3_controller_interaction,
                htc_hand_interaction,
                htc_vive_wrist_tracker_interaction,
                htcx_vive_tracker_interaction,
                huawei_controller_interaction,
                #[cfg(target_os = "android")]
                khr_android_thread_settings,
                #[cfg(target_os = "android")]
                khr_android_surface_swapchain,
                khr_composition_layer_cube,
                #[cfg(target_os = "android")]
                khr_android_create_instance,
                khr_composition_layer_depth,
                khr_vulkan_swapchain_format_list,
                khr_composition_layer_cylinder,
                khr_composition_layer_equirect,
                khr_opengl_enable,
                khr_opengl_es_enable,
                khr_vulkan_enable,
                #[cfg(windows)]
                khr_d3d11_enable,
                #[cfg(windows)]
                khr_d3d12_enable,
                khr_visibility_mask,
                khr_composition_layer_color_scale_bias,
                #[cfg(windows)]
                khr_win32_convert_performance_counter_time,
                khr_convert_timespec_time,
                khr_loader_init,
                #[cfg(target_os = "android")]
                khr_loader_init_android,
                khr_vulkan_enable2,
                khr_composition_layer_equirect2,
                khr_binding_modification,
                khr_swapchain_usage_input_attachment_bit,
                meta_vulkan_swapchain_create_info,
                meta_performance_metrics,
                ml_ml2_controller_interaction,
                mnd_headless,
                mnd_swapchain_usage_input_attachment_bit,
                mndx_egl_enable,
                msft_unbounded_reference_space,
                msft_spatial_anchor,
                msft_spatial_graph_bridge,
                msft_hand_interaction,
                msft_hand_tracking_mesh,
                msft_secondary_view_configuration,
                msft_first_person_observer,
                msft_controller_model,
                #[cfg(windows)]
                msft_perception_anchor_interop,
                #[cfg(windows)]
                msft_holographic_window_attachment,
                msft_composition_layer_reprojection,
                msft_spatial_anchor_persistence,
                #[cfg(target_os = "android")]
                oculus_android_session_state_enable,
                oculus_audio_device_guid,
                ultraleap_hand_tracking_forearm,
                valve_analog_threshold,
                varjo_quad_views,
                varjo_foveated_rendering,
                varjo_composition_layer_depth_test,
                varjo_environment_depth_estimation,
                varjo_marker_tracking,
                varjo_view_offset,
            }
        )*

    };
}

impl_ext!(bitor, bitand);
