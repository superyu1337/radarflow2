use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32, 
    pub y: f32,
    pub z: f32
}

unsafe impl dataview::Pod for Vec3 {}