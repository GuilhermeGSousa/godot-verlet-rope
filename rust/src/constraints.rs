use godot::prelude::*;

use crate::rope_point::RopePoint;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Constraint {
    fn solve(&mut self);
}

pub struct DistanceContraint {
    start: Rc<RefCell<RopePoint>>,
    end: Rc<RefCell<RopePoint>>,
    distance: f32,
}

impl DistanceContraint {
    pub fn new(start: Rc<RefCell<RopePoint>>, end: Rc<RefCell<RopePoint>>, distance: f32) -> Self {
        Self {
            start,
            end,
            distance,
        }
    }
}

impl Constraint for DistanceContraint {
    fn solve(&mut self) {
        let mut start = self.start.borrow_mut();
        let mut end = self.end.borrow_mut();
        let delta_pos = end.position - start.position;
        let current_distance = delta_pos.length();
        let diff = (current_distance - self.distance) / current_distance;
        let correction = 0.5 * diff * delta_pos;

        start.position += correction;
        end.position -= correction;
    }
}

pub struct PinConstraint {
    pin_position: Vector2,
    rope_point: Rc<RefCell<RopePoint>>,
}

impl PinConstraint {
    pub fn new(pin_position: Vector2, rope_point: Rc<RefCell<RopePoint>>) -> Self {
        Self {
            pin_position,
            rope_point,
        }
    }
}

impl Constraint for PinConstraint {
    fn solve(&mut self) {
        let mut rope_point = self.rope_point.borrow_mut();
        rope_point.position = self.pin_position;
    }
}
