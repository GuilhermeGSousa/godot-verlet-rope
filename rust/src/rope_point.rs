use godot::prelude::*;

pub struct RopeParticle {
    pub position: Vector2,
    pub prev_position: Vector2,
}

impl RopeParticle {
    pub fn get_velocity(&self, delta_time: f32) -> Vector2 {
        (self.position - self.prev_position) / delta_time
    }

    pub fn set_velocity(&mut self, velocity: Vector2, delta_time: f32) {
        self.prev_position = self.position - velocity * delta_time;
    }

    pub fn update_position(&mut self, new_position: Vector2) {
        self.prev_position = self.position;
        self.position = new_position;
    }
}
