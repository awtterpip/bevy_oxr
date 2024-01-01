use openxr::ExtensionSet;
use std::ops;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XrExtensions(ExtensionSet);
impl XrExtensions {
    pub fn raw_mut(&mut self) -> &mut ExtensionSet {
        &mut self.0
    }
    pub fn raw(&self) -> &ExtensionSet {
        &self.0
    }
    // pub fn enable_fb_passthrough(&mut self) -> &mut Self {
    //     self.0.fb_passthrough = true;
    //     self
    // }
    // pub fn disable_fb_passthrough(&mut self) -> &mut Self {
    //     self.0.fb_passthrough = false;
    //     self
    // }
    pub fn enable_hand_tracking(&mut self) -> &mut Self {
        self.0.ext_hand_tracking = true;
        self
    }
    pub fn disable_hand_tracking(&mut self) -> &mut Self {
        self.0.ext_hand_tracking = false;
        self
    }
    pub fn enable_local_floor(&mut self) -> &mut Self {
        self.0.ext_local_floor = true;
        self
    }
    pub fn disable_local_floor(&mut self) -> &mut Self {
        self.0.ext_local_floor = false;
        self
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
        exts.ext_local_floor = true;
        Self(exts)
    }
}
impl ops::BitAnd for XrExtensions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut out = ExtensionSet::default();
        out.ext_local_floor = self.0.ext_local_floor && rhs.0.ext_local_floor;
        out.almalence_digital_lens_control =
            self.0.almalence_digital_lens_control && rhs.0.almalence_digital_lens_control;
        out.epic_view_configuration_fov =
            self.0.epic_view_configuration_fov && rhs.0.epic_view_configuration_fov;
        out.ext_performance_settings =
            self.0.ext_performance_settings && rhs.0.ext_performance_settings;
        out.ext_thermal_query = self.0.ext_thermal_query && rhs.0.ext_thermal_query;
        out.ext_debug_utils = self.0.ext_debug_utils && rhs.0.ext_debug_utils;
        out.ext_eye_gaze_interaction =
            self.0.ext_eye_gaze_interaction && rhs.0.ext_eye_gaze_interaction;
        out.ext_view_configuration_depth_range =
            self.0.ext_view_configuration_depth_range && rhs.0.ext_view_configuration_depth_range;
        out.ext_conformance_automation =
            self.0.ext_conformance_automation && rhs.0.ext_conformance_automation;
        out.ext_hand_tracking = self.0.ext_hand_tracking && rhs.0.ext_hand_tracking;
        out.ext_dpad_binding = self.0.ext_dpad_binding && rhs.0.ext_dpad_binding;
        out.ext_hand_joints_motion_range =
            self.0.ext_hand_joints_motion_range && rhs.0.ext_hand_joints_motion_range;
        out.ext_samsung_odyssey_controller =
            self.0.ext_samsung_odyssey_controller && rhs.0.ext_samsung_odyssey_controller;
        out.ext_hp_mixed_reality_controller =
            self.0.ext_hp_mixed_reality_controller && rhs.0.ext_hp_mixed_reality_controller;
        out.ext_palm_pose = self.0.ext_palm_pose && rhs.0.ext_palm_pose;
        out.ext_uuid = self.0.ext_uuid && rhs.0.ext_uuid;
        // out.extx_overlay = self.0.extx_overlay && rhs.0.extx_overlay;
        out.fb_composition_layer_image_layout =
            self.0.fb_composition_layer_image_layout && rhs.0.fb_composition_layer_image_layout;
        out.fb_composition_layer_alpha_blend =
            self.0.fb_composition_layer_alpha_blend && rhs.0.fb_composition_layer_alpha_blend;
        out.fb_swapchain_update_state =
            self.0.fb_swapchain_update_state && rhs.0.fb_swapchain_update_state;
        out.fb_composition_layer_secure_content =
            self.0.fb_composition_layer_secure_content && rhs.0.fb_composition_layer_secure_content;
        out.fb_display_refresh_rate =
            self.0.fb_display_refresh_rate && rhs.0.fb_display_refresh_rate;
        out.fb_color_space = self.0.fb_color_space && rhs.0.fb_color_space;
        out.fb_hand_tracking_mesh = self.0.fb_hand_tracking_mesh && rhs.0.fb_hand_tracking_mesh;
        out.fb_hand_tracking_aim = self.0.fb_hand_tracking_aim && rhs.0.fb_hand_tracking_aim;
        out.fb_hand_tracking_capsules =
            self.0.fb_hand_tracking_capsules && rhs.0.fb_hand_tracking_capsules;
        out.fb_spatial_entity = self.0.fb_spatial_entity && rhs.0.fb_spatial_entity;
        out.fb_foveation = self.0.fb_foveation && rhs.0.fb_foveation;
        out.fb_foveation_configuration =
            self.0.fb_foveation_configuration && rhs.0.fb_foveation_configuration;
        out.fb_keyboard_tracking = self.0.fb_keyboard_tracking && rhs.0.fb_keyboard_tracking;
        out.fb_triangle_mesh = self.0.fb_triangle_mesh && rhs.0.fb_triangle_mesh;
        out.fb_passthrough = self.0.fb_passthrough && rhs.0.fb_passthrough;
        out.fb_render_model = self.0.fb_render_model && rhs.0.fb_render_model;
        out.fb_spatial_entity_query =
            self.0.fb_spatial_entity_query && rhs.0.fb_spatial_entity_query;
        out.fb_spatial_entity_storage =
            self.0.fb_spatial_entity_storage && rhs.0.fb_spatial_entity_storage;
        out.fb_foveation_vulkan = self.0.fb_foveation_vulkan && rhs.0.fb_foveation_vulkan;
        out.fb_swapchain_update_state_opengl_es =
            self.0.fb_swapchain_update_state_opengl_es && rhs.0.fb_swapchain_update_state_opengl_es;
        out.fb_swapchain_update_state_vulkan =
            self.0.fb_swapchain_update_state_vulkan && rhs.0.fb_swapchain_update_state_vulkan;
        out.fb_space_warp = self.0.fb_space_warp && rhs.0.fb_space_warp;
        out.fb_scene = self.0.fb_scene && rhs.0.fb_scene;
        out.fb_spatial_entity_container =
            self.0.fb_spatial_entity_container && rhs.0.fb_spatial_entity_container;
        out.fb_passthrough_keyboard_hands =
            self.0.fb_passthrough_keyboard_hands && rhs.0.fb_passthrough_keyboard_hands;
        out.fb_composition_layer_settings =
            self.0.fb_composition_layer_settings && rhs.0.fb_composition_layer_settings;
        out.htc_vive_cosmos_controller_interaction = self.0.htc_vive_cosmos_controller_interaction
            && rhs.0.htc_vive_cosmos_controller_interaction;
        out.htc_facial_tracking = self.0.htc_facial_tracking && rhs.0.htc_facial_tracking;
        out.htc_vive_focus3_controller_interaction = self.0.htc_vive_focus3_controller_interaction
            && rhs.0.htc_vive_focus3_controller_interaction;
        out.htc_hand_interaction = self.0.htc_hand_interaction && rhs.0.htc_hand_interaction;
        out.htc_vive_wrist_tracker_interaction =
            self.0.htc_vive_wrist_tracker_interaction && rhs.0.htc_vive_wrist_tracker_interaction;
        // out.htcx_vive_tracker_interaction =
        //     self.0.htcx_vive_tracker_interaction && rhs.0.htcx_vive_tracker_interaction;
        out.huawei_controller_interaction =
            self.0.huawei_controller_interaction && rhs.0.huawei_controller_interaction;
        out.khr_composition_layer_cube =
            self.0.khr_composition_layer_cube && rhs.0.khr_composition_layer_cube;
        out.khr_composition_layer_depth =
            self.0.khr_composition_layer_depth && rhs.0.khr_composition_layer_depth;
        out.khr_vulkan_swapchain_format_list =
            self.0.khr_vulkan_swapchain_format_list && rhs.0.khr_vulkan_swapchain_format_list;
        out.khr_composition_layer_cylinder =
            self.0.khr_composition_layer_cylinder && rhs.0.khr_composition_layer_cylinder;
        out.khr_composition_layer_equirect =
            self.0.khr_composition_layer_equirect && rhs.0.khr_composition_layer_equirect;
        out.khr_opengl_enable = self.0.khr_opengl_enable && rhs.0.khr_opengl_enable;
        out.khr_opengl_es_enable = self.0.khr_opengl_es_enable && rhs.0.khr_opengl_es_enable;
        out.khr_vulkan_enable = self.0.khr_vulkan_enable && rhs.0.khr_vulkan_enable;
        out.khr_visibility_mask = self.0.khr_visibility_mask && rhs.0.khr_visibility_mask;
        out.khr_composition_layer_color_scale_bias = self.0.khr_composition_layer_color_scale_bias
            && rhs.0.khr_composition_layer_color_scale_bias;
        out.khr_convert_timespec_time =
            self.0.khr_convert_timespec_time && rhs.0.khr_convert_timespec_time;
        out.khr_loader_init = self.0.khr_loader_init && rhs.0.khr_loader_init;
        out.khr_vulkan_enable2 = self.0.khr_vulkan_enable2 && rhs.0.khr_vulkan_enable2;
        out.khr_composition_layer_equirect2 =
            self.0.khr_composition_layer_equirect2 && rhs.0.khr_composition_layer_equirect2;
        out.khr_binding_modification =
            self.0.khr_binding_modification && rhs.0.khr_binding_modification;
        out.khr_swapchain_usage_input_attachment_bit =
            self.0.khr_swapchain_usage_input_attachment_bit
                && rhs.0.khr_swapchain_usage_input_attachment_bit;
        out.meta_vulkan_swapchain_create_info =
            self.0.meta_vulkan_swapchain_create_info && rhs.0.meta_vulkan_swapchain_create_info;
        out.meta_performance_metrics =
            self.0.meta_performance_metrics && rhs.0.meta_performance_metrics;
        out.ml_ml2_controller_interaction =
            self.0.ml_ml2_controller_interaction && rhs.0.ml_ml2_controller_interaction;
        out.mnd_headless = self.0.mnd_headless && rhs.0.mnd_headless;
        out.mnd_swapchain_usage_input_attachment_bit =
            self.0.mnd_swapchain_usage_input_attachment_bit
                && rhs.0.mnd_swapchain_usage_input_attachment_bit;
        // out.mndx_egl_enable = self.0.mndx_egl_enable && rhs.0.mndx_egl_enable;
        out.msft_unbounded_reference_space =
            self.0.msft_unbounded_reference_space && rhs.0.msft_unbounded_reference_space;
        out.msft_spatial_anchor = self.0.msft_spatial_anchor && rhs.0.msft_spatial_anchor;
        out.msft_spatial_graph_bridge =
            self.0.msft_spatial_graph_bridge && rhs.0.msft_spatial_graph_bridge;
        out.msft_hand_interaction = self.0.msft_hand_interaction && rhs.0.msft_hand_interaction;
        out.msft_hand_tracking_mesh =
            self.0.msft_hand_tracking_mesh && rhs.0.msft_hand_tracking_mesh;
        out.msft_secondary_view_configuration =
            self.0.msft_secondary_view_configuration && rhs.0.msft_secondary_view_configuration;
        out.msft_first_person_observer =
            self.0.msft_first_person_observer && rhs.0.msft_first_person_observer;
        out.msft_controller_model = self.0.msft_controller_model && rhs.0.msft_controller_model;
        out.msft_composition_layer_reprojection =
            self.0.msft_composition_layer_reprojection && rhs.0.msft_composition_layer_reprojection;
        out.msft_spatial_anchor_persistence =
            self.0.msft_spatial_anchor_persistence && rhs.0.msft_spatial_anchor_persistence;
        out.oculus_audio_device_guid =
            self.0.oculus_audio_device_guid && rhs.0.oculus_audio_device_guid;
        out.ultraleap_hand_tracking_forearm =
            self.0.ultraleap_hand_tracking_forearm && rhs.0.ultraleap_hand_tracking_forearm;
        out.valve_analog_threshold = self.0.valve_analog_threshold && rhs.0.valve_analog_threshold;
        out.varjo_quad_views = self.0.varjo_quad_views && rhs.0.varjo_quad_views;
        out.varjo_foveated_rendering =
            self.0.varjo_foveated_rendering && rhs.0.varjo_foveated_rendering;
        out.varjo_composition_layer_depth_test =
            self.0.varjo_composition_layer_depth_test && rhs.0.varjo_composition_layer_depth_test;
        out.varjo_environment_depth_estimation =
            self.0.varjo_environment_depth_estimation && rhs.0.varjo_environment_depth_estimation;
        out.varjo_marker_tracking = self.0.varjo_marker_tracking && rhs.0.varjo_marker_tracking;
        out.varjo_view_offset = self.0.varjo_view_offset && rhs.0.varjo_view_offset;
        and_android_only_exts(&self, &rhs, &mut out);
        and_windows_only_exts(&self, &rhs, &mut out);
        for ext in self.0.other {
            if rhs.0.other.contains(&ext) {
                out.other.push(ext);
            }
        }
        Self(out)
    }
}

#[cfg(not(target_os = "android"))]
fn and_android_only_exts(lhs: &XrExtensions, rhs: &XrExtensions, out: &mut ExtensionSet) {}
#[cfg(not(windows))]
fn and_windows_only_exts(lhs: &XrExtensions, rhs: &XrExtensions, out: &mut ExtensionSet) {}
#[cfg(target_os = "android")]
fn and_android_only_exts(lhs: &XrExtensions, rhs: &XrExtensions, out: &mut ExtensionSet) {
    out.oculus_android_session_state_enable =
        lhs.0.oculus_android_session_state_enable && rhs.0.oculus_android_session_state_enable;
    out.khr_loader_init_android = lhs.0.khr_loader_init_android && rhs.0.khr_loader_init_android;
    out.fb_android_surface_swapchain_create =
        lhs.0.fb_android_surface_swapchain_create && rhs.0.fb_android_surface_swapchain_create;
    out.fb_swapchain_update_state_android_surface = lhs.0.fb_swapchain_update_state_android_surface
        && rhs.0.fb_swapchain_update_state_android_surface;
    out.khr_android_thread_settings =
        lhs.0.khr_android_thread_settings && rhs.0.khr_android_thread_settings;
    out.khr_android_surface_swapchain =
        lhs.0.khr_android_surface_swapchain && rhs.0.khr_android_surface_swapchain;
    out.khr_android_create_instance =
        lhs.0.khr_android_create_instance && rhs.0.khr_android_create_instance;
}
#[cfg(windows)]
fn and_windows_only_exts(lhs: &XrExtensions, rhs: &XrExtensions, out: &mut ExtensionSet) {
    out.ext_win32_appcontainer_compatible =
        lhs.0.ext_win32_appcontainer_compatible && rhs.0.ext_win32_appcontainer_compatible;
    out.khr_d3d11_enable = lhs.0.khr_d3d11_enable && rhs.0.khr_d3d11_enable;
    out.khr_d3d12_enable = lhs.0.khr_d3d12_enable && rhs.0.khr_d3d12_enable;
    out.khr_win32_convert_performance_counter_time =
        lhs.0.khr_win32_convert_performance_counter_time
            && rhs.0.khr_win32_convert_performance_counter_time;
    out.msft_perception_anchor_interop =
        lhs.0.msft_perception_anchor_interop && rhs.0.msft_perception_anchor_interop;
    out.msft_holographic_window_attachment =
        lhs.0.msft_holographic_window_attachment && rhs.0.msft_holographic_window_attachment;
}
