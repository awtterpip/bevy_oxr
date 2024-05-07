use bevy::prelude::*;

pub trait ToPosef {
    fn to_posef(&self) -> openxr::Posef;
}
pub trait ToTransform {
    fn to_transform(&self) -> Transform;
}
pub trait ToQuaternionf {
    fn to_quaternionf(&self) -> openxr::Quaternionf;
}
pub trait ToQuat {
    fn to_quat(&self) -> Quat;
}
pub trait ToVector3f {
    fn to_vector3f(&self) -> openxr::Vector3f;
}
pub trait ToVec3 {
    fn to_vec3(&self) -> Vec3;
}
pub trait ToVector2f {
    fn to_vector2f(&self) -> openxr::Vector2f;
}
pub trait ToVec2 {
    fn to_vec2(&self) -> Vec2;
}
impl ToPosef for Transform {
    fn to_posef(&self) -> openxr::Posef {
        openxr::Posef {
            orientation: self.rotation.to_quaternionf(),
            position: self.translation.to_vector3f(),
        }
    }
}
impl ToTransform for openxr::Posef {
    fn to_transform(&self) -> Transform {
        Transform::from_translation(self.position.to_vec3())
            .with_rotation(self.orientation.to_quat())
    }
}

impl ToQuaternionf for Quat {
    fn to_quaternionf(&self) -> openxr::Quaternionf {
        openxr::Quaternionf {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}
impl ToQuat for openxr::Quaternionf {
    fn to_quat(&self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}
impl ToVector3f for Vec3 {
    fn to_vector3f(&self) -> openxr::Vector3f {
        openxr::Vector3f {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl ToVec3 for openxr::Vector3f {
    fn to_vec3(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl ToVector2f for Vec2 {
    fn to_vector2f(&self) -> openxr::Vector2f {
        openxr::Vector2f {
            x: self.x,
            y: self.y,
        }
    }
}
impl ToVec2 for openxr::Vector2f {
    fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
