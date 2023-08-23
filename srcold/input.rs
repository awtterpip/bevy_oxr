use std::sync::Mutex;

use glam::{Quat, Vec3};
use openxr as xr;

use crate::xr::{VIEW_TYPE, XrPose};

#[derive(Clone)]
pub struct PostFrameData {
    pub views: Vec<xr::View>,
    pub left_hand: Option<XrPose>,
    pub right_hand: Option<XrPose>,
}

pub(crate) struct XrInput {
    session: xr::Session<xr::Vulkan>,
    action_set: xr::ActionSet,
    right_action: xr::Action<xr::Posef>,
    left_action: xr::Action<xr::Posef>,
    right_space: xr::Space,
    left_space: xr::Space,
    stage: xr::Space,
    left_hand: Mutex<XrPose>,
    right_hand: Mutex<XrPose>,
    views: Mutex<Vec<XrPose>>,
}

impl XrInput {
    pub(crate) fn new(
        instance: xr::Instance,
        session: xr::Session<xr::Vulkan>,
    ) -> anyhow::Result<Self> {
        let action_set = instance.create_action_set("input", "input pose information", 0)?;
        let right_action =
            action_set.create_action::<xr::Posef>("right_hand", "Right Hand Controller", &[])?;
        let left_action =
            action_set.create_action::<xr::Posef>("left_hand", "Left Hand Controller", &[])?;
        instance.suggest_interaction_profile_bindings(
            instance.string_to_path("/interaction_profiles/khr/simple_controller")?,
            &[
                xr::Binding::new(
                    &right_action,
                    instance.string_to_path("/user/hand/right/input/grip/pose")?,
                ),
                xr::Binding::new(
                    &left_action,
                    instance.string_to_path("/user/hand/left/input/grip/pose")?,
                ),
            ],
        )?;
        session.attach_action_sets(&[&action_set])?;
        let right_space =
            right_action.create_space(session.clone(), xr::Path::NULL, xr::Posef::IDENTITY)?;
        let left_space =
            left_action.create_space(session.clone(), xr::Path::NULL, xr::Posef::IDENTITY)?;
        let stage =
            session.create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)?;
        Ok(Self {
            left_action,
            left_space,
            right_action,
            right_space,
            action_set,
            stage,
            session,
            left_hand: Default::default(),
            right_hand: Default::default(),
            views: Default::default(),
        })
    }

    pub(crate) fn post_frame(
        &self,
        xr_frame_state: xr::FrameState,
    ) -> xr::Result<PostFrameData> {
        self.session.sync_actions(&[(&self.action_set).into()])?;
        let locate_hand_pose = |action: &xr::Action<xr::Posef>,
                                space: &xr::Space|
         -> xr::Result<Option<(Vec3, Quat)>> {
            if action.is_active(&self.session, xr::Path::NULL)? {
                Ok(Some(openxr_pose_to_glam(
                    &space
                        .locate(&self.stage, xr_frame_state.predicted_display_time)?
                        .pose,
                )))
            } else {
                Ok(None)
            }
        };

        let left_hand = locate_hand_pose(&self.left_action, &self.left_space)?;
        let right_hand = locate_hand_pose(&self.right_action, &self.right_space)?;
        let (_, views) = self.session.locate_views(
            VIEW_TYPE,
            xr_frame_state.predicted_display_time,
            &self.stage,
        )?;

        if let Some(left_hand) = &left_hand {
            *self.left_hand.lock().unwrap() = left_hand.clone();
        }

        if let Some(right_hand) = &left_hand {
            *self.right_hand.lock().unwrap() = right_hand.clone();
        }

        *self.views.lock().unwrap() = views.iter().map(|f| openxr_pose_to_glam(&f.pose)).collect();

        Ok(PostFrameData {
            views,
            left_hand,
            right_hand,
        })
    }

    pub fn stage(&self) -> &xr::Space {
        &self.stage
    }

    pub fn left_hand(&self) -> XrPose {
        *self.left_hand.lock().unwrap()
    }

    pub fn right_hand(&self) -> XrPose {
        *self.right_hand.lock().unwrap()
    }

    pub fn views(&self) -> Vec<XrPose> {
        self.views.lock().unwrap().clone()
    }
}

pub fn openxr_pose_to_glam(pose: &openxr::Posef) -> (Vec3, Quat) {
    // with enough sign errors anything is possible
    let rotation = {
        let o = pose.orientation;
        Quat::from_rotation_x(180.0f32.to_radians()) * glam::quat(o.w, o.z, o.y, o.x)
    };
    let translation = glam::vec3(-pose.position.x, pose.position.y, -pose.position.z);
    (translation, rotation)
}
