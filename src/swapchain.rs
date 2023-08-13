use std::sync::Mutex;

use bevy::prelude::*;
use openxr as xr;



struct Extent2D {
    width: u32,
    height: u32,
}