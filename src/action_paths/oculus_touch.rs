super::actions! {
    "/user"
    hand {
        path: "/hand";
        left {
            path: "/left";
            input {
                path: "/input";
                x {
                    path: "/x";
                    click {
                        path: "/click";
                        name: "x_click";
                        path_type: bool;
                    }
                    touch {
                        path: "/touch";
                        name: "x_touch";
                        path_type: bool;
                    }
                }
                y {
                    path: "/y";
                    click {
                        path: "/click";
                        name: "y_click";
                        path_type: bool;
                    }
                    touch {
                        path: "/touch";
                        name: "y_touch";
                        path_type: bool;
                    }
                }
                menu {
                    path: "/menu";
                    click {
                        path: "/click";
                        name: "menu_click";
                        path_type: bool;
                    }
                }
                squeeze {
                    path: "/squeeze";
                    value {
                        path: "/value";
                        name: "left_grip_val";
                        path_type: f32;
                    }
                }
                trigger {
                    path: "/trigger";
                    value {
                        path: "/value";
                        name: "left_trigger_val";
                        path_type: f32;
                    }
                    touch {
                        path: "/touch";
                        name: "left_trigger_touch";
                        path_type: bool;
                    }
                }
                thumbstick {
                    path: "/thumbstick";
                    x {
                        path: "/x";
                        name: "left_thumbstick_x";
                        path_type: f32;
                    }
                    y {
                        path: "/y";
                        name: "left_thumbstick_y";
                        path_type: f32;
                    }
                    click {
                        path: "/click";
                        name: "left_thumbstick_click";
                        path_type: bool;
                    }
                    touch {
                        path: "/touch";
                        name: "left_thumbstick_touch";
                        path_type: bool;
                    }
                }
                thumbrest {
                    path: "/thumbrest";
                    touch {
                        path: "/touch";
                        name: "left_thumbrest_touch";
                        path_type: bool;
                    }
                }
                grip {
                    path: "/grip";
                    pose {
                        path: "/pose";
                        name: "left_grip_pose";
                        path_type: crate::types::Pose;
                    }
                }
                aim {
                    path: "/aim";
                    pose {
                        path: "/pose";
                        name: "left_aim_pose";
                        path_type: crate::types::Pose;
                    }
                }
            }
            output {
                path: "/output";
                haptic {
                    path: "/haptic";
                    name: "left_controller_haptic";
                    path_type: crate::types::Haptic;
                }
            }
        }
        right {
            path: "/right";
            input {
                path: "/input";
                a {
                    path: "/a";
                    click {
                        path: "/click";
                        name: "a_click";
                        path_type: bool;
                    }
                    touch {
                        path: "/touch";
                        name: "a_touch";
                        path_type: bool;
                    }
                }
                b {
                    path: "/b";
                    click {
                        path: "/click";
                        name: "b_click";
                        path_type: bool;
                    }
                    touch {
                        path: "/touch";
                        name: "b_touch";
                        path_type: bool;
                    }
                }
                system {
                    path: "/system";
                    click {
                        path: "/click";
                        name: "system_click";
                        path_type: bool;
                    }
                }
                squeeze {
                    path: "/squeeze";
                    value {
                        path: "/value";
                        name: "right_grip_val";
                        path_type: f32;
                    }
                }
                trigger {
                    path: "/trigger";
                    value {
                        path: "/value";
                        name: "right_trigger_val";
                        path_type: f32;
                    }
                    touch {
                        path: "/touch";
                        name: "right_trigger_touch";
                        path_type: bool;
                    }
                }
                thumbstick {
                    path: "/thumbstick";
                    x {
                        path: "/x";
                        name: "right_thumbstick_x";
                        path_type: f32;
                    }
                    y {
                        path: "/y";
                        name: "right_thumbstick_y";
                        path_type: f32;
                    }
                    click {
                        path: "/click";
                        name: "right_thumbstick_click";
                        path_type: bool;
                    }
                    touch {
                        path: "/touch";
                        name: "right_thumbstick_touch";
                        path_type: bool;
                    }
                }
                thumbrest {
                    path: "/thumbrest";
                    touch {
                        path: "/touch";
                        name: "right_thumbrest_touch";
                        path_type: bool;
                    }
                }
                grip {
                    path: "/grip";
                    pose {
                        path: "/pose";
                        name: "right_grip_pose";
                        path_type: crate::types::Pose;
                    }
                }
                aim {
                    path: "/aim";
                    pose {
                        path: "/pose";
                        name: "right_aim_pose";
                        path_type: crate::types::Pose;
                    }
                }
            }
            output {
                path: "/output";
                haptic {
                    path: "/haptic";
                    name: "right_controller_haptic";
                    path_type: crate::types::Haptic;
                }
            }
        }
    }
}
