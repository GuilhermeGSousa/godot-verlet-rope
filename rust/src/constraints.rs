use godot::prelude::*;

use crate::rope_point::RopeParticle;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub trait Constraint {
    fn solve(&mut self);
}

pub struct DistanceContraint {
    start: Weak<RefCell<RopeParticle>>,
    end: Weak<RefCell<RopeParticle>>,
    distance: f32,
}

impl DistanceContraint {
    pub fn new(
        start: Weak<RefCell<RopeParticle>>,
        end: Weak<RefCell<RopeParticle>>,
        distance: f32,
    ) -> Self {
        Self {
            start,
            end,
            distance,
        }
    }
}

impl Constraint for DistanceContraint {
    fn solve(&mut self) {
        match (self.start.upgrade(), self.end.upgrade()) {
            (Some(start), Some(end)) => {
                let mut start = start.borrow_mut();
                let mut end = end.borrow_mut();
                let delta_pos = end.position - start.position;
                let current_distance = delta_pos.length();
                let diff = (current_distance - self.distance) / current_distance;
                let correction = 0.5 * diff * delta_pos;

                start.position += correction;
                end.position -= correction;
            }
            _ => (),
        };
    }
}

pub struct PinConstraint {
    pin_position: Vector2,
    rope_point: Rc<RefCell<RopeParticle>>,
}

impl PinConstraint {
    pub fn new(pin_position: Vector2, rope_point: Rc<RefCell<RopeParticle>>) -> Self {
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

pub struct Node2DPinContraint {
    rope_point: Weak<RefCell<RopeParticle>>,
    node: Gd<Node2D>,
}

impl Node2DPinContraint {
    pub fn new(rope_point: Weak<RefCell<RopeParticle>>, node: Gd<Node2D>) -> Self {
        Self { rope_point, node }
    }
}

impl Constraint for Node2DPinContraint {
    fn solve(&mut self) {
        if let Some(rope_point) = self.rope_point.upgrade() {
            let pin_position = self.node.get_position();
            rope_point.borrow_mut().position = pin_position;
        }
    }
}
