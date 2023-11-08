use bevy::prelude::{Quat, Transform, Vec3};
use openxr::{Posef, Quaternionf, Vector3f};

use super::Hand;

pub fn get_simulated_open_hand_transforms(hand: Hand) -> [Transform; 26] {
    let test_hand_bones: [Vec3; 26] = [
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }, //palm
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: -0.04,
        }, //wrist
        Vec3 {
            x: -0.02,
            y: 0.00,
            z: 0.015,
        }, //thumb
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.03,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.024,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.024,
        },
        Vec3 {
            x: -0.01,
            y: -0.015,
            z: 0.0155,
        }, //index
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.064,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.037,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.02,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.01,
        },
        Vec3 {
            x: 0.0,
            y: -0.02,
            z: 0.016,
        }, //middle
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.064,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.037,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.02,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.01,
        },
        Vec3 {
            x: 0.01,
            y: -0.015,
            z: 0.015,
        }, //ring
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.064,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.037,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.02,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.01,
        },
        Vec3 {
            x: 0.02,
            y: -0.01,
            z: 0.015,
        }, //little
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.064,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.037,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.02,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.01,
        },
    ];
    let result = bones_to_transforms(test_hand_bones, hand);
    result
}

fn bones_to_transforms(hand_bones: [Vec3; 26], hand: Hand) -> [Transform; 26] {
    match hand {
        Hand::Left => {
            let mut result_array: [Transform; 26] = [Transform::default(); 26];
            for (place, data) in result_array.iter_mut().zip(hand_bones.iter()) {
                *place = Transform {
                    translation: Vec3 {
                        x: -data.x,
                        y: -data.y,
                        z: -data.z,
                    },
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(1.0),
                }
            }
            result_array
        }
        Hand::Right => {
            let mut result_array: [Transform; 26] = [Transform::default(); 26];
            for (place, data) in result_array.iter_mut().zip(hand_bones.iter()) {
                *place = Transform {
                    translation: Vec3 {
                        x: data.x,
                        y: -data.y,
                        z: -data.z,
                    },
                    rotation: Quat::IDENTITY,
                    scale: Vec3::splat(1.0),
                }
            }
            result_array
        }
    }
}

pub fn get_test_hand_pose_array() -> [Posef; 26] {
    let test_hand_pose: [Posef; 26] = [
        Posef {
            position: Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            orientation: Quaternionf {
                x: -0.267,
                y: 0.849,
                z: 0.204,
                w: 0.407,
            },
        }, //palm
        Posef {
            position: Vector3f {
                x: 0.02,
                y: -0.040,
                z: -0.015,
            },
            orientation: Quaternionf {
                x: -0.267,
                y: 0.849,
                z: 0.204,
                w: 0.407,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.019,
                y: -0.037,
                z: 0.011,
            },
            orientation: Quaternionf {
                x: -0.744,
                y: -0.530,
                z: 0.156,
                w: -0.376,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.015,
                y: -0.014,
                z: 0.047,
            },
            orientation: Quaternionf {
                x: -0.786,
                y: -0.550,
                z: 0.126,
                w: -0.254,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.004,
                y: 0.003,
                z: 0.068,
            },
            orientation: Quaternionf {
                x: -0.729,
                y: -0.564,
                z: 0.027,
                w: -0.387,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.009,
                y: 0.011,
                z: 0.072,
            },
            orientation: Quaternionf {
                x: -0.585,
                y: -0.548,
                z: -0.140,
                w: -0.582,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.027,
                y: -0.021,
                z: 0.001,
            },
            orientation: Quaternionf {
                x: -0.277,
                y: -0.826,
                z: 0.317,
                w: -0.376,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.002,
                y: 0.026,
                z: 0.034,
            },
            orientation: Quaternionf {
                x: -0.277,
                y: -0.826,
                z: 0.317,
                w: -0.376,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.023,
                y: 0.049,
                z: 0.055,
            },
            orientation: Quaternionf {
                x: -0.244,
                y: -0.843,
                z: 0.256,
                w: -0.404,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.037,
                y: 0.059,
                z: 0.067,
            },
            orientation: Quaternionf {
                x: -0.200,
                y: -0.866,
                z: 0.165,
                w: -0.428,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.045,
                y: 0.063,
                z: 0.073,
            },
            orientation: Quaternionf {
                x: -0.172,
                y: -0.874,
                z: 0.110,
                w: -0.440,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.021,
                y: -0.017,
                z: -0.007,
            },
            orientation: Quaternionf {
                x: -0.185,
                y: -0.817,
                z: 0.370,
                w: -0.401,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.011,
                y: 0.029,
                z: 0.018,
            },
            orientation: Quaternionf {
                x: -0.185,
                y: -0.817,
                z: 0.370,
                w: -0.401,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.034,
                y: 0.06,
                z: 0.033,
            },
            orientation: Quaternionf {
                x: -0.175,
                y: -0.809,
                z: 0.371,
                w: -0.420,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.051,
                y: 0.072,
                z: 0.045,
            },
            orientation: Quaternionf {
                x: -0.109,
                y: -0.856,
                z: 0.245,
                w: -0.443,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.06,
                y: 0.077,
                z: 0.051,
            },
            orientation: Quaternionf {
                x: -0.075,
                y: -0.871,
                z: 0.180,
                w: -0.450,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.013,
                y: -0.017,
                z: -0.015,
            },
            orientation: Quaternionf {
                x: -0.132,
                y: -0.786,
                z: 0.408,
                w: -0.445,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.02,
                y: 0.025,
                z: 0.0,
            },
            orientation: Quaternionf {
                x: -0.132,
                y: -0.786,
                z: 0.408,
                w: -0.445,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.042,
                y: 0.055,
                z: 0.007,
            },
            orientation: Quaternionf {
                x: -0.131,
                y: -0.762,
                z: 0.432,
                w: -0.464,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.06,
                y: 0.069,
                z: 0.015,
            },
            orientation: Quaternionf {
                x: -0.071,
                y: -0.810,
                z: 0.332,
                w: -0.477,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.069,
                y: 0.075,
                z: 0.02,
            },
            orientation: Quaternionf {
                x: -0.029,
                y: -0.836,
                z: 0.260,
                w: -0.482,
            },
        },
        Posef {
            position: Vector3f {
                x: 0.004,
                y: -0.022,
                z: -0.022,
            },
            orientation: Quaternionf {
                x: -0.060,
                y: -0.749,
                z: 0.481,
                w: -0.452,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.028,
                y: 0.018,
                z: -0.015,
            },
            orientation: Quaternionf {
                x: -0.060,
                y: -0.749,
                z: 0.481,
                w: -0.452,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.046,
                y: 0.042,
                z: -0.017,
            },
            orientation: Quaternionf {
                x: -0.061,
                y: -0.684,
                z: 0.534,
                w: -0.493,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.059,
                y: 0.053,
                z: -0.015,
            },
            orientation: Quaternionf {
                x: 0.002,
                y: -0.745,
                z: 0.444,
                w: -0.498,
            },
        },
        Posef {
            position: Vector3f {
                x: -0.068,
                y: 0.059,
                z: -0.013,
            },
            orientation: Quaternionf {
                x: 0.045,
                y: -0.780,
                z: 0.378,
                w: -0.496,
            },
        },
    ];
    test_hand_pose
}
