use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32, 
    pub y: f32,
    pub z: f32
}

unsafe impl dataview::Pod for Vec3 {}