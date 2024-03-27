use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ToolPreviewResource {
    pub tool_coordinates: Vec2,
    pub tool_color: Vec3,
    pub tool_radius: f32,
}
