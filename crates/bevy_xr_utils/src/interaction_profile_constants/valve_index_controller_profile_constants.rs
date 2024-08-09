#![allow(dead_code, non_snake_case)]
use std::borrow::Cow;
//here be a large block of ugly strings

//left hand
//system button
const LEFT_HAND_SYSTEM_BUTTON_CLICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/system/click");
const LEFT_HAND_SYSTEM_BUTTON_TOUCH: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/system/touch");
//a button
const LEFT_HAND_A_BUTTON_CLICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/a/click");
const LEFT_HAND_A_BUTTON_TOUCH: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/a/touch");
//b button
const LEFT_HAND_B_BUTTON_CLICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/b/click");
const LEFT_HAND_B_BUTTON_TOUCH: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/b/touch");
//squeeze
const LEFT_HAND_SQUEEZE_VALUE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/squeeze/value",
);
const LEFT_HAND_SQUEEZE_FORCE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/squeeze/force",
);
//trigger
const LEFT_HAND_TRIGGER_CLICK: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/trigger/click",
);
const LEFT_HAND_TRIGGER_VALUE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/trigger/value",
);
const LEFT_HAND_TRIGGER_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/trigger/touch",
);
//thumbstick
const LEFT_HAND_THUMBSTICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/thumbstick");
const LEFT_HAND_THUMBSTICK_X: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/thumbstick/x");
const LEFT_HAND_THUMBSTICK_Y: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/thumbstick/y");
const LEFT_HAND_THUMBSTICK_CLICK: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/thumbstick/click",
);
const LEFT_HAND_THUMBSTICK_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/thumbstick/touch",
);
//trackpad
const LEFT_HAND_TRACKPAD_X: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/trackpad/x");
const LEFT_HAND_TRACKPAD_Y: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/trackpad/y");
const LEFT_HAND_TRACKPAD_FORCE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/trackpad/force",
);
const LEFT_HAND_TRACKPAD_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/left/input/trackpad/touch",
);
//grip
const LEFT_HAND_GRIP_POSE: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/grip/pose");
//aim
const LEFT_HAND_AIM_POSE: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/left/input/aim/pose");

//right hand
//system button
const RIGHT_HAND_SYSTEM_BUTTON_CLICK: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/system/click",
);
const RIGHT_HAND_SYSTEM_BUTTON_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/system/touch",
);
//a button
const RIGHT_HAND_A_BUTTON_CLICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/a/click");
const RIGHT_HAND_A_BUTTON_TOUCH: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/a/touch");
//b button
const RIGHT_HAND_B_BUTTON_CLICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/b/click");
const RIGHT_HAND_B_BUTTON_TOUCH: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/b/touch");
//squeeze
const RIGHT_HAND_SQUEEZE_VALUE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/squeeze/value",
);
const RIGHT_HAND_SQUEEZE_FORCE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/squeeze/force",
);
//trigger
const RIGHT_HAND_TRIGGER_CLICK: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/trigger/click",
);
const RIGHT_HAND_TRIGGER_VALUE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/trigger/value",
);
const RIGHT_HAND_TRIGGER_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/RIrightGHT/input/trigger/touch",
);
//thumbstick
const RIGHT_HAND_THUMBSTICK: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/thumbstick");
const RIGHT_HAND_THUMBSTICK_X: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/thumbstick/x",
);
const RIGHT_HAND_THUMBSTICK_Y: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/thumbstick/y",
);
const RIGHT_HAND_THUMBSTICK_CLICK: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/thumbstick/click",
);
const RIGHT_HAND_THUMBSTICK_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/thumbstick/touch",
);
//trackpad
const RIGHT_HAND_TRACKPAD_X: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/trackpad/x");
const RIGHT_HAND_TRACKPAD_Y: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/trackpad/y");
const RIGHT_HAND_TRACKPAD_FORCE: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/trackpad/force",
);
const RIGHT_HAND_TRACKPAD_TOUCH: Cow<'static, str> = Cow::Borrowed(
    "/interaction_profiles/valve/index_controller/user/hand/right/input/trackpad/touch",
);
//grip
const RIGHT_HAND_GRIP_POSE: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/grip/pose");
//aim
const RIGHT_HAND_AIM_POSE: Cow<'static, str> =
    Cow::Borrowed("/interaction_profiles/valve/index_controller/user/hand/right/input/aim/pose");

//structs
pub struct ButtonProfile<'a> {
    pub TOUCH: Cow<'a, str>,
    pub CLICK: Cow<'a, str>,
}

pub struct SqueezeProfile<'a> {
    pub VALUE: Cow<'a, str>,
    pub FORCE: Cow<'a, str>,
}

pub struct TriggerProfile<'a> {
    pub CLICK: Cow<'a, str>,
    pub VALUE: Cow<'a, str>,
    pub TOUCH: Cow<'a, str>,
}

pub struct ThumbstickProfile<'a> {
    pub X: Cow<'a, str>,
    pub Y: Cow<'a, str>,
    pub CLICK: Cow<'a, str>,
    pub TOUCH: Cow<'a, str>,
}

pub struct TrackpadProfile<'a> {
    pub X: Cow<'a, str>,
    pub Y: Cow<'a, str>,
    pub FORCE: Cow<'a, str>,
    pub TOUCH: Cow<'a, str>,
}

pub struct SpaceProfile<'a> {
    POSE: Cow<'a, str>,
}
pub struct Side<'a> {
    pub INPUT: IndexInput<'a>,
}

pub struct IndexInput<'a> {
    pub SYSTEM: ButtonProfile<'a>,
    pub A: ButtonProfile<'a>,
    pub B: ButtonProfile<'a>,
    pub SQUEEZE: SqueezeProfile<'a>,
    pub TRIGGER: TriggerProfile<'a>,
    pub THUMSTICK: ThumbstickProfile<'a>,
    pub TRACKPAD: TrackpadProfile<'a>,
    pub GRIP: SpaceProfile<'a>,
    pub AIM: SpaceProfile<'a>,
}

pub struct Hand<'a> {
    pub LEFT: Side<'a>,
    pub RIGHT: Side<'a>,
}

pub struct User<'a> {
    HAND: Hand<'a>,
}

pub struct Profile<'a> {
    USER: User<'a>,
}

//const structs?
//LEFT
//system button
const LEFT_SYSTEM_BUTTON: ButtonProfile<'static> = ButtonProfile {
    CLICK: LEFT_HAND_SYSTEM_BUTTON_CLICK,
    TOUCH: LEFT_HAND_SYSTEM_BUTTON_TOUCH,
};
//a button
const LEFT_A_BUTTON: ButtonProfile<'static> = ButtonProfile {
    CLICK: LEFT_HAND_A_BUTTON_CLICK,
    TOUCH: LEFT_HAND_A_BUTTON_TOUCH,
};
//b button
const LEFT_B_BUTTON: ButtonProfile<'static> = ButtonProfile {
    CLICK: LEFT_HAND_B_BUTTON_CLICK,
    TOUCH: LEFT_HAND_B_BUTTON_TOUCH,
};
//squeeze
const LEFT_SQUEEZE: SqueezeProfile<'static> = SqueezeProfile {
    VALUE: LEFT_HAND_SQUEEZE_VALUE,
    FORCE: LEFT_HAND_SQUEEZE_FORCE,
};
//trigger
const LEFT_TRIGGER: TriggerProfile<'static> = TriggerProfile {
    CLICK: LEFT_HAND_TRIGGER_CLICK,
    VALUE: LEFT_HAND_TRIGGER_VALUE,
    TOUCH: LEFT_HAND_TRIGGER_VALUE,
};
//thumbstick
const LEFT_THUMBSTICK: ThumbstickProfile<'static> = ThumbstickProfile {
    X: LEFT_HAND_THUMBSTICK_X,
    Y: LEFT_HAND_THUMBSTICK_Y,
    CLICK: LEFT_HAND_THUMBSTICK_CLICK,
    TOUCH: LEFT_HAND_THUMBSTICK_TOUCH,
};
//trackpad
const LEFT_TRACKPAD: TrackpadProfile<'static> = TrackpadProfile {
    X: LEFT_HAND_TRACKPAD_X,
    Y: LEFT_HAND_TRACKPAD_Y,
    FORCE: LEFT_HAND_TRACKPAD_FORCE,
    TOUCH: LEFT_HAND_TRACKPAD_TOUCH,
};
//grip
const LEFT_GRIP: SpaceProfile<'static> = SpaceProfile {
    POSE: LEFT_HAND_GRIP_POSE,
};
//aim
const LEFT_AIM: SpaceProfile<'static> = SpaceProfile {
    POSE: LEFT_HAND_AIM_POSE,
};

const LEFT_INPUT: IndexInput<'static> = IndexInput {
    SYSTEM: LEFT_SYSTEM_BUTTON,
    A: LEFT_A_BUTTON,
    B: LEFT_B_BUTTON,
    SQUEEZE: LEFT_SQUEEZE,
    TRIGGER: LEFT_TRIGGER,
    THUMSTICK: LEFT_THUMBSTICK,
    TRACKPAD: LEFT_TRACKPAD,
    GRIP: LEFT_GRIP,
    AIM: LEFT_AIM,
};

const LEFT_HAND: Side<'static> = Side { INPUT: LEFT_INPUT };

//RIGHT
//system button
const RIGHT_SYSTEM_BUTTON: ButtonProfile<'static> = ButtonProfile {
    CLICK: RIGHT_HAND_SYSTEM_BUTTON_CLICK,
    TOUCH: RIGHT_HAND_SYSTEM_BUTTON_TOUCH,
};
//a button
const RIGHT_A_BUTTON: ButtonProfile<'static> = ButtonProfile {
    CLICK: RIGHT_HAND_A_BUTTON_CLICK,
    TOUCH: RIGHT_HAND_A_BUTTON_TOUCH,
};
//b button
const RIGHT_B_BUTTON: ButtonProfile<'static> = ButtonProfile {
    CLICK: RIGHT_HAND_B_BUTTON_CLICK,
    TOUCH: RIGHT_HAND_B_BUTTON_TOUCH,
};
//squeeze
const RIGHT_SQUEEZE: SqueezeProfile<'static> = SqueezeProfile {
    VALUE: RIGHT_HAND_SQUEEZE_VALUE,
    FORCE: RIGHT_HAND_SQUEEZE_FORCE,
};
//trigger
const RIGHT_TRIGGER: TriggerProfile<'static> = TriggerProfile {
    CLICK: RIGHT_HAND_TRIGGER_CLICK,
    VALUE: RIGHT_HAND_TRIGGER_VALUE,
    TOUCH: RIGHT_HAND_TRIGGER_VALUE,
};
//thumbstick
const RIGHT_THUMBSTICK: ThumbstickProfile<'static> = ThumbstickProfile {
    X: RIGHT_HAND_THUMBSTICK_X,
    Y: RIGHT_HAND_THUMBSTICK_Y,
    CLICK: RIGHT_HAND_THUMBSTICK_CLICK,
    TOUCH: RIGHT_HAND_THUMBSTICK_TOUCH,
};
//trackpad
const RIGHT_TRACKPAD: TrackpadProfile<'static> = TrackpadProfile {
    X: RIGHT_HAND_TRACKPAD_X,
    Y: RIGHT_HAND_TRACKPAD_Y,
    FORCE: RIGHT_HAND_TRACKPAD_FORCE,
    TOUCH: RIGHT_HAND_TRACKPAD_TOUCH,
};
//grip
const RIGHT_GRIP: SpaceProfile<'static> = SpaceProfile {
    POSE: RIGHT_HAND_GRIP_POSE,
};
//aim
const RIGHT_AIM: SpaceProfile<'static> = SpaceProfile {
    POSE: RIGHT_HAND_AIM_POSE,
};

const RIGHT_INPUT: IndexInput<'static> = IndexInput {
    SYSTEM: RIGHT_SYSTEM_BUTTON,
    A: RIGHT_A_BUTTON,
    B: RIGHT_B_BUTTON,
    SQUEEZE: RIGHT_SQUEEZE,
    TRIGGER: RIGHT_TRIGGER,
    THUMSTICK: RIGHT_THUMBSTICK,
    TRACKPAD: RIGHT_TRACKPAD,
    GRIP: RIGHT_GRIP,
    AIM: RIGHT_AIM,
};

const RIGHT_HAND: Side<'static> = Side { INPUT: RIGHT_INPUT };
//
const HAND: Hand<'static> = Hand {
    LEFT: LEFT_HAND,
    RIGHT: RIGHT_HAND,
};

const USER: User<'static> = User { HAND: HAND };

const INDEX_CONTROLLER: Profile<'static> = Profile { USER: USER };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_a_click_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/left/input/a/click";
        assert_eq!(LEFT_A_BUTTON.CLICK, test_value);
    }

    #[test]
    fn left_hand_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/left/input/a/click";
        assert_eq!(LEFT_HAND.INPUT.A.CLICK, test_value);
    }

    #[test]
    fn hand_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/left/input/a/click";
        assert_eq!(HAND.LEFT.INPUT.A.CLICK, test_value);
    }

    #[test]
    fn user_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/left/input/a/click";
        assert_eq!(USER.HAND.LEFT.INPUT.A.CLICK, test_value);
    }

    #[test]
    fn index_controller_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/left/input/a/click";
        assert_eq!(INDEX_CONTROLLER.USER.HAND.LEFT.INPUT.A.CLICK, test_value);
    }

    #[test]
    fn index_controller_left_trigger_click_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/left/input/trigger/click";
        assert_eq!(
            INDEX_CONTROLLER.USER.HAND.LEFT.INPUT.TRIGGER.CLICK,
            test_value
        );
    }

    #[test]
    fn index_controller_right_trigger_click_test() {
        let test_value =
            "/interaction_profiles/valve/index_controller/user/hand/right/input/trigger/click";
        assert_eq!(
            INDEX_CONTROLLER.USER.HAND.RIGHT.INPUT.TRIGGER.CLICK,
            test_value
        );
    }
}
