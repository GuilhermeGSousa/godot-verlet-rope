use crate::constraints::{Constraint, DistanceContraint, PinConstraint};
use crate::rope_engine::RopeEngine;
use crate::rope_point::RopePoint;
use godot::engine::{Line2D, Node2D, ProjectSettings};
use godot::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Rope2D {
    #[base]
    base: Base<Node2D>,

    #[export]
    line_renderer: Option<Gd<Line2D>>,

    pub rope_particles: Vec<Rc<RefCell<RopePoint>>>,
    constraints: Vec<Box<dyn Constraint>>,
    segment_distance: f32,
    // engine specific
    gravity_vector: Vector2,
    gravity: f32,
    iteration_count: i32,
}

#[godot_api]
impl Rope2D {
    fn get_rope_engine(&self) -> Gd<RopeEngine> {
        self.base
            .get_node_as::<RopeEngine>("/root/GlobalRopeEngine")
    }

    fn initialize(&mut self) {
        self.rope_particles.clear();
        for point in self.line_renderer.as_ref().unwrap().get_points().as_slice() {
            self.rope_particles.push(Rc::new(RefCell::new(RopePoint {
                position: point.clone(),
                prev_position: point.clone(),
            })));
        }

        for i in 0..self.rope_particles.len() - 1 {
            self.constraints.push(Box::new(DistanceContraint::new(
                self.rope_particles[i].clone(),
                self.rope_particles[i + 1].clone(),
                self.segment_distance,
            )));
        }
        let first_rope_point = self.rope_particles[0].clone();
        let pin_position: Vector2 = first_rope_point.borrow().position;
        self.constraints
            .push(Box::new(PinConstraint::new(pin_position, first_rope_point)))
    }

    fn render(&mut self) {
        let line_renderer: &mut Gd<Line2D> = self.line_renderer.as_mut().unwrap();

        line_renderer.clear_points();

        for p in &self.rope_particles {
            line_renderer.add_point(p.borrow().position);
        }
    }
}

#[godot_api]
impl Node2DVirtual for Rope2D {
    fn init(node2d: Base<Node2D>) -> Self {
        let project_settings = ProjectSettings::singleton();
        Self {
            base: node2d,
            line_renderer: None,
            rope_particles: Vec::new(),
            constraints: Vec::new(),
            segment_distance: 50.0,
            gravity_vector: project_settings
                .get_setting("physics/2d/default_gravity_vector".into())
                .to(),
            gravity: project_settings
                .get_setting("physics/2d/default_gravity".into())
                .to(),
            iteration_count: 50,
        }
    }

    fn enter_tree(&mut self) {
        self.initialize();
        let _engine = self.get_rope_engine();
        // Register
    }
    fn exit_tree(&mut self) {
        let _engine = self.get_rope_engine();
        // Unresgister
    }
    fn ready(&mut self) {}

    fn physics_process(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        let gravity_displacement =
            0.5 * delta_time * delta_time * self.gravity * self.gravity_vector;

        // Apply Gravity
        for p in &mut self.rope_particles {
            let velocity = p.borrow().get_velocity(delta_time);
            let current_position =
                p.borrow().position + velocity * delta_time + gravity_displacement;
            p.borrow_mut().update_position(current_position);
        }

        for _ in 0..self.iteration_count {
            for constraint in &mut self.constraints {
                // Solve
                constraint.solve();
            }
        }

        self.render();
    }
}
